pub mod ci;
pub mod common;
pub mod catalog;
pub mod docker;
pub mod git;
pub mod python;
pub mod rust;
pub mod sql;
pub mod typescript;

pub use catalog::{
    ConfidencePolicy, Language, LanguageFinding, LanguageRule, Matcher, ProofWindow,
};

use crate::audit::helpers::AuditContext;

pub fn catalog() -> &'static [catalog::LanguageRule] {
    catalog::all()
}

pub fn findings(ctx: &AuditContext) -> Vec<LanguageFinding> {
    let mut out = Vec::new();
    out.extend(rust::findings(ctx));
    out.extend(sql::findings(ctx));
    out.extend(typescript::findings(ctx));
    out.extend(docker::findings(ctx));
    out.extend(python::findings(ctx));
    out.extend(ci::findings(ctx));
    out.extend(git::findings(ctx));
    out
}
