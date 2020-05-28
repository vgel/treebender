#[macro_use]
extern crate lazy_static;

pub mod featurestructure;
pub mod earley;
pub mod rules;
pub mod syntree;
pub mod forest;
pub mod grammar;

use earley::parse_chart;


pub type Err = Box<dyn std::error::Error + 'static>;

const GRAMMAR: &str = r#"
    S -> N[ case: nom; num: #1; ] IV[ num: #1 ]
    S -> N[ case: nom; pron: #1; num: #2 ] TV[ num: #2 ] N[ case: acc, needs_pron: #1 ]
    S -> N[ case: nom; num: #1; ] CV[ num: #1 ] Comp S

    N[ num: sg, case: nom, pron: he ]         -> he
    N[ num: sg, case: acc, pron: he ]         -> him
    N[ num: sg, case: acc, needs_pron: he ]   -> himself
    N[ num: sg, case: nom, pron: she ]        -> she
    N[ num: sg, case: acc, pron: she ]        -> her
    N[ num: sg, case: acc, needs_pron: she]   -> herself
    N[ num: pl, case: nom, pron: they ]       -> they
    N[ num: pl, case: acc, pron: they ]       -> them
    N[ num: pl, case: acc, needs_pron: they ] -> themselves
    N[ num: sg, pron: she ] -> Mary
    N[ num: sg, pron: she ] -> Sue
    N[ num: sg, pron: he ]  -> Takeshi
    N[ num: sg, pron: he ]  -> Robert

    IV[ num:  sg, tense: nonpast ] -> falls
    IV[ num:  pl, tense: nonpast ] -> fell
    IV[ num: top, tense: past ]    -> fell

    TV[ num:  sg, tense: nonpast ] -> kisses
    TV[ num:  pl, tense: nonpast ] -> kiss
    TV[ num: top: tense: past ]    -> kissed

    TV[ num:  sg, tense: nonpast ] -> says
    TV[ num:  pl, tense: nonpast ] -> say
    TV[ num: top, tense: past ]    -> said

"#;

fn main() {
    let _ = GRAMMAR;
    //let fs = featurestructure::FeatureStructure::new();
}
