/*!
A symbolic natural language parsing library for Rust, inspired by
[HDPSG](https://en.wikipedia.org/wiki/Head-driven_phrase_structure_grammar).

# What is this?
This is a library for parsing natural or constructed languages into syntax trees
and feature structures. There's no machine learning or probabilistic models,
everything is hand-crafted and deterministic.

You can find out more about the motivations of this project in
[this blog post](https://vgel.me/posts/symbolic-linguistics-part1/).

## But what are you using it for?
I'm using this to parse a constructed language for my upcoming xenolinguistics
game, [Themengi](https://vgel.me/themengi/).

# Motivation
Using a simple 80-line grammar, introduced in the tutorial below, we can parse
a simple subset of English, checking reflexive pronoun binding, case, and
number agreement.

```text
$ cargo run --bin cli examples/reflexives.fgr
> she likes himself
Parsed 0 trees

> her likes herself
Parsed 0 trees

> she like herself
Parsed 0 trees

> she likes herself
Parsed 1 tree
(0..3: S
  (0..1: N (0..1: she))
  (1..2: TV (1..2: likes))
  (2..3: N (2..3: herself)))
[
  child-2: [
    case: acc
    pron: ref
    needs_pron: #0 she
    num: sg
    child-0: [ word: herself ]
  ]
  child-1: [
    tense: nonpast
    child-0: [ word: likes ]
    num: #1 sg
  ]
  child-0: [
    child-0: [ word: she ]
    case: nom
    pron: #0
    num: #1
  ]
]
```

Low resource language? Low problem! No need to train on gigabytes of text, just
write a grammar using your brain. Let's hypothesize that in
American Sign Language, topicalized nouns (expressed with raised eyebrows)
must appear first in the sentence. We can write a small grammar (18 lines),
and plug in some sentences:

```text
$ cargo run --bin cli examples/asl-wordorder.fgr -n
> boy sit
Parsed 1 tree
(0..2: S
  (0..1: NP ((0..1: N (0..1: boy))))
  (1..2: IV (1..2: sit)))

> boy throw ball
Parsed 1 tree
(0..3: S
  (0..1: NP ((0..1: N (0..1: boy))))
  (1..2: TV (1..2: throw))
  (2..3: NP ((2..3: N (2..3: ball)))))

> ball nm-raised-eyebrows boy throw
Parsed 1 tree
(0..4: S
  (0..2: NP
    (0..1: N (0..1: ball))
    (1..2: Topic (1..2: nm-raised-eyebrows)))
  (2..3: NP ((2..3: N (2..3: boy))))
  (3..4: TV (3..4: throw)))

> boy throw ball nm-raised-eyebrows
Parsed 0 trees
```

# Tutorial
As an example, let's say we want to build a parser for English reflexive
pronouns (himself, herself, themselves, themself, itself). We'll also support
number ("He likes X" v.s. "They like X") and simple embedded clauses
("He said that they like X").

Grammar files are written in a custom language, similar to BNF, called
Feature GRammar (.fgr). There's a VSCode syntax highlighting extension for these
files available as [`fgr-syntax`](https://marketplace.visualstudio.com/items?itemName=vgel.fgr-syntax).

We'll start by defining our lexicon. The lexicon is the set of terminal symbols
(symbols in the actual input) that the grammar will match. Terminal symbols must
start with a lowercase letter, and non-terminal symbols must start with an
uppercase letter.

```fgr
// pronouns
N -> he
N -> him
N -> himself
N -> she
N -> her
N -> herself
N -> they
N -> them
N -> themselves
N -> themself

// names, lowercase as they are terminals
N -> mary
N -> sue
N -> takeshi
N -> robert

// complementizer
Comp -> that

// verbs -- intransitive, transitive, and clausal
IV -> falls
IV -> fall
IV -> fell

TV -> likes
TV -> like
TV -> liked

CV -> says
CV -> say
CV -> said
```

Next, we can add our sentence rules (they must be added at the top, as the first
rule in the file is assumed to be the top-level rule):

```fgr
// sentence rules
S -> N IV
S -> N TV N
S -> N CV Comp S

// ... previous lexicon ...
```

Assuming this file is saved as `examples/no-features.fgr` (which it is :wink:),
we can test this file with the built-in CLI:

```text
$ cargo run --bin cli examples/no-features.fgr
> he falls
Parsed 1 tree
(0..2: S
  (0..1: N (0..1: he))
  (1..2: IV (1..2: falls)))
[
  child-1: [ child-0: [ word: falls ] ]
  child-0: [ child-0: [ word: he ] ]
]

> he falls her
Parsed 0 trees

> he likes her
Parsed 1 tree
(0..3: S
  (0..1: N (0..1: he))
  (1..2: TV (1..2: likes))
  (2..3: N (2..3: her)))
[
  child-2: [ child-0: [ word: her ] ]
  child-1: [ child-0: [ word: likes ] ]
  child-0: [ child-0: [ word: he ] ]
]

> he likes
Parsed 0 trees

> he said that he likes her
Parsed 1 tree
(0..6: S
  (0..1: N (0..1: he))
  (1..2: CV (1..2: said))
  (2..3: Comp (2..3: that))
  (3..6: S
    (3..4: N (3..4: he))
    (4..5: TV (4..5: likes))
    (5..6: N (5..6: her))))
[
  child-0: [ child-0: [ word: he ] ]
  child-2: [ child-0: [ word: that ] ]
  child-1: [ child-0: [ word: said ] ]
  child-3: [
    child-2: [ child-0: [ word: her ] ]
    child-1: [ child-0: [ word: likes ] ]
    child-0: [ child-0: [ word: he ] ]
  ]
]

> he said that he
Parsed 0 trees
```

This grammar already parses some correct sentences, and blocks some trivially
incorrect ones. However, it doesn't care about number, case, or reflexives
right now:

```text
> she likes himself  // unbound reflexive pronoun
Parsed 1 tree
(0..3: S
  (0..1: N (0..1: she))
  (1..2: TV (1..2: likes))
  (2..3: N (2..3: himself)))
[
  child-0: [ child-0: [ word: she ] ]
  child-2: [ child-0: [ word: himself ] ]
  child-1: [ child-0: [ word: likes ] ]
]

> him like her  // incorrect case on the subject pronoun, should be nominative
                // (he) instead of accusative (him)
Parsed 1 tree
(0..3: S
  (0..1: N (0..1: him))
  (1..2: TV (1..2: like))
  (2..3: N (2..3: her)))
[
  child-0: [ child-0: [ word: him ] ]
  child-1: [ child-0: [ word: like ] ]
  child-2: [ child-0: [ word: her ] ]
]

> he like her  // incorrect verb number agreement
Parsed 1 tree
(0..3: S
  (0..1: N (0..1: he))
  (1..2: TV (1..2: like))
  (2..3: N (2..3: her)))
[
  child-2: [ child-0: [ word: her ] ]
  child-1: [ child-0: [ word: like ] ]
  child-0: [ child-0: [ word: he ] ]
]
```

To fix this, we need to add *features* to our lexicon, and restrict the sentence
rules based on features.

Features are added with square brackets, and are key: value pairs separated by
commas. `**top**` is a special feature value, which basically means
"unspecified" -- we'll come back to it later. Features that are unspecified are
also assumed to have a `**top**` value, but sometimes explicitly stating top is
more clear.

```fgr
/// Pronouns
// The added features are:
// * num: sg or pl, whether this noun wants a singular verb (likes) or
//   a plural verb (like). note this is grammatical number, so for example
//   singular they takes plural agreement ("they like X", not *"they likes X")
// * case: nom or acc, whether this noun is nominative or accusative case.
//   nominative case goes in the subject, and accusative in the object.
//   e.g., "he fell" and "she likes him", not *"him fell" and *"her likes he"
// * pron: he, she, they, or ref -- what type of pronoun this is
// * needs_pron: whether this is a reflexive that needs to bind to another
//   pronoun.
N[ num: sg, case: nom, pron: he ]                    -> he
N[ num: sg, case: acc, pron: he ]                    -> him
N[ num: sg, case: acc, pron: ref, needs_pron: he ]   -> himself
N[ num: sg, case: nom, pron: she ]                   -> she
N[ num: sg, case: acc, pron: she ]                   -> her
N[ num: sg, case: acc, pron: ref, needs_pron: she]   -> herself
N[ num: pl, case: nom, pron: they ]                  -> they
N[ num: pl, case: acc, pron: they ]                  -> them
N[ num: pl, case: acc, pron: ref, needs_pron: they ] -> themselves
N[ num: sg, case: acc, pron: ref, needs_pron: they ] -> themself

// Names
// The added features are:
// * num: sg, as people are singular ("mary likes her" / *"mary like her")
// * case: **top**, as names can be both subjects and objects
//   ("mary likes her" / "she likes mary")
// * pron: whichever pronoun the person uses for reflexive agreement
//   mary    pron: she  => mary likes herself
//   sue     pron: they => sue likes themself
//   takeshi pron: he   => takeshi likes himself
N[ num: sg, case: **top**, pron: she ]  -> mary
N[ num: sg, case: **top**, pron: they ] -> sue
N[ num: sg, case: **top**, pron: he ]   -> takeshi
N[ num: sg, case: **top**, pron: he ]   -> robert

// Complementizer doesn't need features
Comp -> that

// Verbs -- intransitive, transitive, and clausal
// The added features are:
// * num: sg, pl, or **top** -- to match the noun numbers.
//   **top** will match either sg or pl, as past-tense verbs in English
//   don't agree in number: "he fell" and "they fell" are both fine
// * tense: past or nonpast -- this won't be used for agreement, but will be
//   copied into the final feature structure, and the client code could do
//   something with it
IV[ num:      sg, tense: nonpast ] -> falls
IV[ num:      pl, tense: nonpast ] -> fall
IV[ num: **top**, tense: past ]    -> fell

TV[ num:      sg, tense: nonpast ] -> likes
TV[ num:      pl, tense: nonpast ] -> like
TV[ num: **top**, tense: past ]    -> liked

CV[ num:      sg, tense: nonpast ] -> says
CV[ num:      pl, tense: nonpast ] -> say
CV[ num: **top**, tense: past ]    -> said
```

Now that our lexicon is updated with features, we can update our sentence rules
to constrain parsing based on those features. This uses two new features,
tags and unification. Tags allow features to be associated between nodes in a
rule, and unification controls how those features are compatible. The rules for
unification are:

1. A string feature can unify with a string feature with the same value
2. A **top** feature can unify with anything, and the nodes are merged
3. A complex feature ([ ... ] structure) is recursively unified with another
   complex feature.

If unification fails anywhere, the parse is aborted and the tree is discarded.
This allows the programmer to discard trees if features don't match.

```fgr
// Sentence rules
// Intransitive verb:
// * Subject must be nominative case
// * Subject and verb must agree in number (copied through #1)
S -> N[ case: nom, num: #1 ] IV[ num: #1 ]
// Transitive verb:
// * Subject must be nominative case
// * Subject and verb must agree in number (copied through #2)
// * If there's a reflexive in the object position, make sure its `needs_pron`
//   feature matches the subject's `pron` feature. If the object isn't a
//   reflexive, then its `needs_pron` feature will implicitly be `**top**`, so
//   will unify with anything.
S -> N[ case: nom, pron: #1, num: #2 ] TV[ num: #2 ] N[ case: acc, needs_pron: #1 ]
// Clausal verb:
// * Subject must be nominative case
// * Subject and verb must agree in number (copied through #1)
// * Reflexives can't cross clause boundaries (*"He said that she likes himself"),
//   so we can ignore reflexives and delegate to inner clause rule
S -> N[ case: nom, num: #1 ] CV[ num: #1 ] Comp S
```

Now that we have this augmented grammar (available as `examples/reflexives.fgr`),
we can try it out and see that it rejects illicit sentences that were previously
accepted, while still accepting valid ones:

```text
> he fell
Parsed 1 tree
(0..2: S
  (0..1: N (0..1: he))
  (1..2: IV (1..2: fell)))
[
  child-1: [
    child-0: [ word: fell ]
    num: #0 sg
    tense: past
  ]
  child-0: [
    pron: he
    case: nom
    num: #0
    child-0: [ word: he ]
  ]
]

> he like him
Parsed 0 trees

> he likes himself
Parsed 1 tree
(0..3: S
  (0..1: N (0..1: he))
  (1..2: TV (1..2: likes))
  (2..3: N (2..3: himself)))
[
  child-1: [
    num: #0 sg
    child-0: [ word: likes ]
    tense: nonpast
  ]
  child-2: [
    needs_pron: #1 he
    num: sg
    child-0: [ word: himself ]
    pron: ref
    case: acc
  ]
  child-0: [
    child-0: [ word: he ]
    pron: #1
    num: #0
    case: nom
  ]
]

> he likes herself
Parsed 0 trees

> mary likes herself
Parsed 1 tree
(0..3: S
  (0..1: N (0..1: mary))
  (1..2: TV (1..2: likes))
  (2..3: N (2..3: herself)))
[
  child-0: [
    pron: #0 she
    num: #1 sg
    case: nom
    child-0: [ word: mary ]
  ]
  child-1: [
    tense: nonpast
    child-0: [ word: likes ]
    num: #1
  ]
  child-2: [
    child-0: [ word: herself ]
    num: sg
    pron: ref
    case: acc
    needs_pron: #0
  ]
]

> mary likes themself
Parsed 0 trees

> sue likes themself
Parsed 1 tree
(0..3: S
  (0..1: N (0..1: sue))
  (1..2: TV (1..2: likes))
  (2..3: N (2..3: themself)))
[
  child-0: [
    pron: #0 they
    child-0: [ word: sue ]
    case: nom
    num: #1 sg
  ]
  child-1: [
    tense: nonpast
    num: #1
    child-0: [ word: likes ]
  ]
  child-2: [
    needs_pron: #0
    case: acc
    pron: ref
    child-0: [ word: themself ]
    num: sg
  ]
]

> sue likes himself
Parsed 0 trees
```

If this is interesting to you and you want to learn more, you can check out
[my blog series](https://vgel.me/posts/symbolic-linguistics-part1/),
the excellent textbook [Syntactic Theory: A Formal Introduction (2nd ed.)](https://web.stanford.edu/group/cslipublications/cslipublications/site/1575864002.shtml),
and the [DELPH-IN project](http://www.delph-in.net/wiki/index.php/Home), whose
work on the LKB inspired this simplified version.

# Using from code
I need to write this section in more detail, but if you're comfortable with Rust,
I suggest looking through the codebase. It's not perfect, it started as one of
my first Rust projects (after migrating through F# -> TypeScript -> C in search
of the right performance/ergonomics tradeoff), and it could use more tests,
but overall it's not too bad.

Basically, the processing pipeline is:

1. Make a `Grammar` struct
  * `Grammar` is defined in `rules.rs`.
  * The easiest way to make a `Grammar` is `Grammar::parse_from_file`, which is
    mostly a hand-written recusive descent parser in `parse_grammar.rs`. Yes,
    I recognize the irony here.
2. It takes input (in `Grammar::parse`, which does everything for you, or
   `Grammar::parse_chart`, which just does the chart)
3. The input is first chart-parsed in `earley.rs`
4. Then, a forest is built from the chart, in `forest.rs`, using an algorithm
    I found in a very useful blog series I forget the URL for, because the
    algorithms in the academic literature for this are... weird.
5. Finally, the feature unification is used to prune the forest down to only
   valid trees. It would be more efficient to do this during parsing, but meh.

The most interesting thing you can do via code and not via the CLI is probably
getting at the raw feature DAG, as that would let you do things like pronoun
coreference. The DAG code is in `featurestructure.rs`, and should be fairly
approachable -- there's a lot of Rust ceremony around `Rc<RefCell<...>>`
because using an arena allocation crate seemed ~~too har~~like overkill, but
that is somewhat mitigated by the `NodeRef` type alias. Hit me up at
https://vgel.me/contact if you need help with anything here!
*/

#[macro_use]
extern crate lazy_static;

pub mod earley;
pub mod featurestructure;
pub mod fgr;
pub mod forest;
pub mod rules;
pub mod syntree;
pub mod utils;

use std::fs;
use std::path;
use std::sync::Arc;

pub use crate::earley::{parse_chart, Chart};
pub use crate::featurestructure::NodeRef;
pub use crate::forest::Forest;
pub use crate::rules::{Grammar, Rule};
pub use crate::syntree::{Constituent, SynTree};
pub use crate::utils::Err;

impl Grammar {
  pub fn parse_chart(&self, input: &[&str]) -> Chart {
    parse_chart(self, input)
  }

  pub fn parse_forest(&self, input: &[&str]) -> Forest {
    Forest::from(self.parse_chart(input))
  }

  pub fn unify_tree(
    tree: SynTree<Arc<Rule>, String>,
  ) -> Result<(SynTree<String, String>, NodeRef), Err> {
    match tree {
      SynTree::Leaf(w) => Ok((SynTree::Leaf(w), NodeRef::new_top())),
      SynTree::Branch(cons, children) => {
        let features = cons.value.features.deep_clone();

        let mut bare_children = Vec::with_capacity(children.len());
        for (idx, child) in children.into_iter().enumerate() {
          let (child_tree, child_features) = Self::unify_tree(child)?;
          bare_children.push(child_tree);

          let to_unify = NodeRef::new_with_edges(vec![(format!("child-{}", idx), child_features)])?;
          NodeRef::unify(features.clone(), to_unify)?;
        }

        let bare_self = SynTree::Branch(
          Constituent {
            span: cons.span,
            value: cons.value.symbol.clone(),
          },
          bare_children,
        );

        Ok((bare_self, features))
      }
    }
  }

  pub fn parse(&self, input: &[&str]) -> Vec<(SynTree<String, String>, NodeRef)> {
    let forest = self.parse_forest(input);
    let trees = forest.trees(self);
    trees
      .into_iter()
      .filter_map(|t| Self::unify_tree(t).map(Some).unwrap_or(None))
      .collect::<Vec<_>>()
  }

  pub fn read_from_file<P: AsRef<path::Path>>(path: P) -> Result<Self, Err> {
    fs::read_to_string(path)?.parse()
  }
}

#[test]
fn test_unification_blocking() {
  let g: Grammar = r#"
    S -> N[ case: nom, pron: #1 ] TV N[ case: acc, needs_pron: #1 ]
    TV -> likes
    N[ case: nom, pron: she ] -> she
    N[ case: nom, pron: he ] -> he
    N[ case: acc, pron: he ] -> him
    N[ case: acc, pron: ref, needs_pron: he ] -> himself
  "#
  .parse()
  .unwrap();

  assert_eq!(g.parse(&["he", "likes", "himself"]).len(), 1);
  assert_eq!(g.parse(&["he", "likes", "him"]).len(), 1);
  assert_eq!(g.parse(&["she", "likes", "him"]).len(), 1);

  assert_eq!(g.parse(&["himself", "likes", "himself"]).len(), 0);
  assert_eq!(g.parse(&["she", "likes", "himself"]).len(), 0);
  assert_eq!(g.parse(&["himself", "likes", "him"]).len(), 0);
}
