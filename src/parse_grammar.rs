/// Simple recursive-descent parsing of grammar files
use regex::Regex;
use std::collections::HashSet;

use crate::featurestructure::NodeRef;
use crate::rules::{Production, Rule, Symbol};
use crate::Err;

pub const TOP_STR: &str = "**top**";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParsedFeatureValue {
  Top,
  Str(String),
}

// use instead of FromStr because it can't fail
impl From<&str> for ParsedFeatureValue {
  /// Returns Top if s == TOP_STR, else allocates a String for Str
  fn from(s: &str) -> Self {
    if s == TOP_STR {
      Self::Top
    } else {
      Self::Str(s.to_string())
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedFeature {
  pub dotted: String,
  pub tag: Option<String>,
  pub value: ParsedFeatureValue,
}

#[derive(Debug, Clone, PartialEq)]
// We can't know whether this is a symbol or terminal before parsing all
// the rules and learning all the left-hand symbols
pub struct ParsedSymbolOrTerminal {
  pub name: String,
  pub features: Vec<ParsedFeature>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParsedRule {
  pub symbol: ParsedSymbolOrTerminal,
  pub productions: Vec<ParsedSymbolOrTerminal>,
}

/// Parses a str into a tuple of (rules, nonterminals)
/// Errors if the grammar doesn't parse or is malformed
pub fn parse(s: &str) -> Result<(Vec<Rule>, HashSet<String>), Err> {
  let rules = parse_rules(s)?;
  let nonterminals: HashSet<String> = rules.iter().map(|r| r.symbol.name.to_string()).collect();
  let rules: Vec<Rule> = rules
    .into_iter()
    .map(|r| make_rule(r, &nonterminals))
    .collect::<Result<_, Err>>()?;
  Ok((rules, nonterminals))
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
fn parse_feature_value(s: &str) -> ParseResult<(Option<String>, ParsedFeatureValue)> {
  regex_static!(VALUE, r"[a-zA-Z0-9\-_\*]+");
  let (tag, s) = parse_tag(s)?;
  let s = skip_whitespace(s);
  let (name, s) = optional_re(&*VALUE, s);
  let value = if let Some(name) = name {
    ParsedFeatureValue::from(name)
  } else if tag.is_some() {
    ParsedFeatureValue::Top
  } else {
    return Err(format!("feature needs tag or value at {}", s).into());
  };
  Ok(((tag, value), s))
}

fn parse_feature(s: &str) -> ParseResult<ParsedFeature> {
  let (name, s) = parse_dotted(s).map_err(|e| format!("feature name: {}", e))?;
  let s = skip_whitespace(s);
  let (_, s) = needed_char(':', s)?;
  let s = skip_whitespace(s);
  let (value, s) = parse_feature_value(s).map_err(|e| format!("feature value: {}", e))?;
  let s = skip_whitespace(s);
  let (_, s) = optional_char(',', s);

  Ok((
    ParsedFeature {
      dotted: name.to_string(),
      tag: value.0,
      value: value.1,
    },
    s,
  ))
}

fn parse_featurestructure(s: &str) -> ParseResult<Vec<ParsedFeature>> {
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

/// name, features (maybe empty)
fn parse_symbol(s: &str) -> ParseResult<ParsedSymbolOrTerminal> {
  let (name, s) = parse_name(s).map_err(|e| -> Err { format!("symbol: {}", e).into() })?;
  let s = skip_whitespace(s);
  let (features, s) = if s.starts_with('[') {
    parse_featurestructure(s)?
  } else {
    (Vec::new(), s)
  };

  Ok((
    ParsedSymbolOrTerminal {
      name: name.to_string(),
      features,
    },
    s,
  ))
}

/// Symbol, productions, terminated by final newline
fn parse_rule(s: &str) -> ParseResult<ParsedRule> {
  #![allow(clippy::trivial_regex)]
  regex_static!(ARROW, "->");

  let (symbol, s) = parse_symbol(s).map_err(|e| -> Err { format!("rule symbol: {}", e).into() })?;
  let s = skip_whitespace(s);
  let (_, s) = needed_re(&*ARROW, s).map_err(|e| -> Err { format!("rule arrow: {}", e).into() })?;
  let mut productions = Vec::new();
  let mut rem = s;
  loop {
    rem = skip_whitespace(rem);
    if let (Some(_), s) = optional_char(';', rem) {
      return Ok((
        ParsedRule {
          symbol,
          productions,
        },
        s,
      ));
    }
    let (prod, s) =
      parse_symbol(rem).map_err(|e| -> Err { format!("rule production: {}", e).into() })?;
    productions.push(prod);
    rem = s;
  }
}

/// We want rules to be able to access their child features, and to be able to
/// unify between them
/// So we have the rule symbol "adopt" the features of its children, copying the
/// child features into child-0.(...), child-1.(...), etc.
///
/// We could try to implement this when constructing the rule, but it's easier
/// to do as a simple AST transform.
fn adopt_child_features(r: ParsedRule) -> ParsedRule {
  let mut symbol_features = r.symbol.features;
  symbol_features.extend(
    r.productions
      .iter()
      .enumerate()
      .map(|(idx, prod)| {
        prod.features.iter().map(move |feature| {
          let name = format!("child-{}.", idx) + &feature.dotted;
          ParsedFeature {
            dotted: name,
            tag: feature.tag.clone(),
            value: feature.value.clone(),
          }
        })
      })
      .flatten(),
  );
  ParsedRule {
    symbol: ParsedSymbolOrTerminal {
      name: r.symbol.name,
      features: symbol_features,
    },
    productions: r.productions,
  }
}

fn parse_rules(s: &str) -> Result<Vec<ParsedRule>, Err> {
  let mut rules = Vec::new();
  let mut rem = s;
  loop {
    rem = skip_whitespace(rem);
    if rem.is_empty() {
      return Ok(rules);
    }
    let (rule, s) = parse_rule(rem)?;
    let rule = adopt_child_features(rule);
    rules.push(rule);
    rem = s;
  }
}

fn make_symbol(s: ParsedSymbolOrTerminal) -> Result<Symbol, Err> {
  let features = s.features.into_iter().collect::<Vec<_>>();
  let symbol = Symbol::new(s.name, NodeRef::new_from_paths(features)?);
  Ok(symbol)
}

fn make_production(
  s: ParsedSymbolOrTerminal,
  rule_name: &str,
  nonterminals: &HashSet<String>,
) -> Result<Production, Err> {
  if nonterminals.contains(&s.name) {
    let symbol = make_symbol(s)?;
    Ok(Production::Nonterminal(symbol))
  } else if s.features.is_empty() {
    Ok(Production::Terminal(s.name))
  } else {
    Err(
      format!(
        "rule {}: production {}: can't have features on terminal!",
        rule_name, s.name
      )
      .into(),
    )
  }
}

fn make_rule(r: ParsedRule, nonterminals: &HashSet<String>) -> Result<Rule, Err> {
  let rule_name = r.symbol.name.clone();
  let productions: Vec<Production> = r
    .productions
    .into_iter()
    .map(|s| make_production(s, &rule_name, &nonterminals))
    .collect::<Result<_, Err>>()?;
  let symbol = make_symbol(r.symbol)?;
  Ok(Rule {
    symbol,
    productions,
  })
}
