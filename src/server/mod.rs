mod r#const;
mod r#fn;
mod r#impl;
mod r#struct;

pub use {r#const::*, r#fn::*, r#struct::*};

#[allow(unused_imports)]
use {super::*, r#impl::*};
