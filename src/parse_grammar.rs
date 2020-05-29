use regex::Regex;
/// Simple recursive-descent parsing of grammar files
use std::str::FromStr;

use crate::featurestructure::{Feature, NodeRef};
use crate::rules::{Grammar, Production, Rule, Symbol};
use crate::Err;

pub const TOP_STR: &str = "**top**";

/// Parses a str into a tuple of (rules, nonterminals)
/// Errors if the grammar doesn't parse or is malformed
impl FromStr for Grammar {
  type Err = Err;

  /// Parses a grammar from a string. Assumes the first rule's symbol
  /// is the start symbol.
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let (rules, s) = parse_rules(s)?;
    assert!(s.is_empty());

    if rules.is_empty() {
      Err("empty ruleset".into())
    } else {
      Ok(Self::new(rules))
    }
  }
}

type Infallible<'a, T> = (T, &'a str);
type ParseResult<'a, T> = Result<(T, &'a str), Err>;

/// helper macro for initializing a regex with lazy_static!
macro_rules! regex_static {
  ($name:ident, $pattern:expr) => {
    lazy_static! {
      static ref $name: Regex = Regex::new($pattern).unwrap();
    }
  };
}

/// Try to consume a regex, returning None if it doesn't match
fn optional_re<'a>(re: &'static Regex, s: &'a str) -> Infallible<'a, Option<&'a str>> {
  if let Some(caps) = re.captures(s) {
    let m = caps.get(0).unwrap();
    if m.start() > 0 {
      return (None, s);
    }
    let (_, rest) = s.split_at(m.end());
    (Some(m.as_str()), rest)
  } else {
    (None, s)
  }
}

/// Try to consume a regex, failing if it doesn't match
fn needed_re<'a>(re: &'static Regex, s: &'a str) -> ParseResult<'a, &'a str> {
  if let (Some(c), rest) = optional_re(re, s) {
    Ok((c, rest))
  } else {
    Err(format!("couldn't match {} at {}", re, s).into())
  }
}

/// Try to consume a char, returning None if it doesn't match
fn optional_char(c: char, s: &str) -> Infallible<Option<char>> {
  let mut iter = s.char_indices().peekable();
  if let Some((_, c1)) = iter.next() {
    if c == c1 {
      let rest = if let Some((idx, _)) = iter.peek() {
        s.split_at(*idx).1
      } else {
        ""
      };
      return (Some(c), rest);
    }
  }
  (None, s)
}

/// Try to consume a char, failing if it doesn't match
fn needed_char(c: char, s: &str) -> ParseResult<char> {
  if let (Some(c), rest) = optional_char(c, s) {
    Ok((c, rest))
  } else {
    Err(format!("couldn't match {} at {}", c, s).into())
  }
}

/// Tries to skip 1 or more \s characters and comments
fn skip_whitespace(s: &str) -> &str {
  regex_static!(WHITESPACE_OR_COMMENT, r"\s+(//.*?\n\s+)*");
  optional_re(&*WHITESPACE_OR_COMMENT, s).1
}

/// Tries to parse a name made of letters, numbers, - and _
fn parse_name(s: &str) -> ParseResult<&str> {
  regex_static!(NAME, r"[a-zA-Z0-9\-_]+");
  needed_re(&*NAME, s).map_err(|err| format!("name: {}", err).into())
}

/// Tries to parse a name made of dotted segments (foo.bar.c.d)
fn parse_dotted(s: &str) -> ParseResult<&str> {
  regex_static!(DOTTED, r"[a-zA-Z0-9\-_]+(\.[a-zA-Z0-9\-_]+)*");
  needed_re(&*DOTTED, s).map_err(|e| format!("dotted name: {}", e).into())
}

/// Parses an optional #tag
fn parse_tag(s: &str) -> ParseResult<Option<String>> {
  let (hash, s) = optional_char('#', s);
  if hash.is_none() {
    Ok((None, s))
  } else {
    let s = skip_whitespace(s);
    let (name, s) = parse_name(s).map_err(|e| -> Err { format!("tag: {}", e).into() })?;
    Ok((Some(name.to_string()), s))
  }
}

/// Parses a value with an optional tag: #tag value
fn parse_feature_value(s: &str) -> ParseResult<(Option<String>, NodeRef)> {
  regex_static!(VALUE, r"[a-zA-Z0-9\-_\*]+");
  let (tag, s) = parse_tag(s)?;
  let s = skip_whitespace(s);
  let (name, s) = optional_re(&*VALUE, s);
  let value = if let Some(name) = name {
    NodeRef::new_str(name.to_string())
  } else if tag.is_some() {
    NodeRef::new_top()
  } else {
    return Err(format!("feature needs tag or value at {}", s).into());
  };
  Ok(((tag, value), s))
}

fn parse_feature(s: &str) -> ParseResult<Feature> {
  let (name, s) = parse_dotted(s).map_err(|e| format!("feature name: {}", e))?;
  let s = skip_whitespace(s);
  let (_, s) = needed_char(':', s)?;
  let s = skip_whitespace(s);
  let (value, s) = parse_feature_value(s).map_err(|e| format!("feature value: {}", e))?;
  let s = skip_whitespace(s);
  let (_, s) = optional_char(',', s);

  Ok((
    Feature {
      path: name.to_string(),
      tag: value.0,
      value: value.1,
    },
    s,
  ))
}

fn parse_featurestructure(s: &str) -> ParseResult<Vec<Feature>> {
  let mut pairs = Vec::new();
  let mut rem = needed_char('[', s)?.1;
  loop {
    rem = skip_whitespace(rem);
    if let (Some(_), rem) = optional_char(']', rem) {
      return Ok((pairs, rem));
    }
    let (feature, s) = parse_feature(rem)?;
    pairs.push(feature);
    rem = s;
  }
}

fn parse_production(s: &str) -> ParseResult<(Production, Vec<Feature>)> {
  let (name, s) = parse_name(s).map_err(|e| -> Err { format!("symbol: {}", e).into() })?;
  let s = skip_whitespace(s);
  let (features, s) = if s.starts_with('[') {
    parse_featurestructure(s)?
  } else {
    (Vec::new(), s)
  };

  if name.chars().next().unwrap().is_lowercase() {
    if !features.is_empty() {
      Err(format!("terminal (lower-case) cannot have features: {} {}", name, s).into())
    } else {
      // annotate terminals with their matching string
      Ok((
        (
          Production::Terminal(name.to_string()),
          vec![Feature {
            path: "word".to_string(),
            tag: None,
            value: NodeRef::new_str(name.to_string()),
          }],
        ),
        s,
      ))
    }
  } else {
    Ok((
      (
        Production::Nonterminal(Symbol {
          name: name.to_string(),
        }),
        features,
      ),
      s,
    ))
  }
}

fn parse_symbol(s: &str) -> ParseResult<(Symbol, Vec<Feature>)> {
  let (prod, s) = parse_production(s)?;
  match prod {
    (Production::Nonterminal(symbol), features) => Ok(((symbol, features), s)),
    (Production::Terminal(w), _) => {
      Err(format!("expected symbol, got terminal {}: {}", w, s).into())
    }
  }
}

/// Symbol, productions, terminated by final newline
fn parse_rule(s: &str) -> ParseResult<Rule> {
  #![allow(clippy::trivial_regex)]
  regex_static!(ARROW, "->");

  let ((symbol, features), s) =
    parse_symbol(s).map_err(|e| -> Err { format!("rule symbol: {}", e).into() })?;
  let s = skip_whitespace(s);
  let (_, s) = needed_re(&*ARROW, s).map_err(|e| -> Err { format!("rule arrow: {}", e).into() })?;

  let mut prods_features = Vec::new();
  let mut rem = s;
  loop {
    rem = skip_whitespace(rem);
    if let (Some(_), s) = optional_char(';', rem) {
      rem = s;
      break;
    }
    let (prod, s) =
      parse_production(rem).map_err(|e| -> Err { format!("rule production: {}", e).into() })?;
    prods_features.push(prod);
    rem = s;
  }

  let (features, productions) = adopt_child_features(features, prods_features);
  let features = NodeRef::new_from_paths(features)?;

  Ok((
    Rule {
      symbol,
      features,
      productions,
    },
    rem,
  ))
}

/// We want rules to be able to access their child features, and to be able to
/// unify between them
/// So we have the rule symbol "adopt" the features of its children, copying the
/// child features into child-0.(...), child-1.(...), etc.
///
/// We could try to implement this when constructing the rule, but it's easier
/// to do as a simple AST transform.
fn adopt_child_features(
  mut rule_features: Vec<Feature>,
  prods_features: Vec<(Production, Vec<Feature>)>,
) -> (Vec<Feature>, Vec<Production>) {
  let mut productions = Vec::with_capacity(prods_features.len());

  for (idx, (prod, features)) in prods_features.into_iter().enumerate() {
    productions.push(prod);
    let prefix = format!("child-{}.", idx);
    for feature in features.into_iter() {
      rule_features.push(Feature {
        path: prefix.clone() + &feature.path,
        tag: feature.tag,
        value: feature.value,
      });
    }
  }

  (rule_features, productions)
}

fn parse_rules(s: &str) -> ParseResult<Vec<Rule>> {
  let mut rules = Vec::new();
  let mut rem = s;
  loop {
    rem = skip_whitespace(rem);
    if rem.is_empty() {
      return Ok((rules, rem));
    }
    let (rule, s) = parse_rule(rem)?;
    rules.push(rule);
    rem = s;
  }
}
