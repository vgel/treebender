#[macro_use]
extern crate lazy_static;

pub mod featurestructure;
pub mod earley;
pub mod rules;
pub mod syntree;
pub mod forest;
pub mod grammar;
pub mod parse_grammar;

use grammar::Grammar;

pub type Err = Box<dyn std::error::Error + 'static>;

const GRAMMAR: &str = r#"
    // sentence rules
    S -> N[ case: nom, num: #1 ] IV[ num: #1 ];  // verb must agree with subject in number
    // if there's a reflexive in the object position, make sure it is filled by the subject
    // also constrain number as before
    S -> N[ case: nom, pron: #1, num: #2 ] TV[ num: #2 ] N[ case: acc, needs_pron: #1 ];
    // reflexives can't cross clause boundaries so just constrain num
    S -> N[ case: nom, num: #1 ] CV[ num: #1 ] Comp S;

    // pronouns
    // reflexives are pron: ref to block * himself kissed himself
    N[ num: sg, case: nom, pron: he ]         -> he;
    N[ num: sg, case: acc, pron: he ]         -> him;
    N[ num: sg, case: acc, pron: ref, needs_pron: he ]   -> himself;
    N[ num: sg, case: nom, pron: she ]        -> she;
    N[ num: sg, case: acc, pron: she ]        -> her;
    N[ num: sg, case: acc, pron: ref, needs_pron: she]   -> herself;
    N[ num: pl, case: nom, pron: they ]       -> they;
    N[ num: pl, case: acc, pron: they ]       -> them;
    N[ num: pl, case: acc, pron: ref, needs_pron: they ] -> themselves;
    // names are case-agnostic
    N[ num: sg, pron: she ] -> mary;
    N[ num: sg, pron: she ] -> sue;
    N[ num: sg, pron: he ]  -> takeshi;
    N[ num: sg, pron: he ]  -> robert;

    Comp -> that;

    IV[ num:      sg, tense: nonpast ] -> falls;
    IV[ num:      pl, tense: nonpast ] -> fell;
    IV[ num: **top**, tense: past ]    -> fell;

    TV[ num:      sg, tense: nonpast ] -> kisses;
    TV[ num:      pl, tense: nonpast ] -> kiss;
    TV[ num: **top**, tense: past ]    -> kissed;

    CV[ num:      sg, tense: nonpast ] -> says;
    CV[ num:      pl, tense: nonpast ] -> say;
    CV[ num: **top**, tense: past ]    -> said;

"#;

fn main() -> Result<(), Err> {
    let g: Grammar = GRAMMAR.parse()?;

    let sentence = "He said that Sue kissed Mary".to_ascii_lowercase();
    let sentence = sentence.split(' ').collect::<Vec<_>>();

    let trees = g.parse(&sentence);

    println!("Parsed {} tree{}", trees.len(), if trees.len() == 1 { "" } else { "s" });
    for t in trees {
        println!("{}", t);
    }

    Ok(())
}
