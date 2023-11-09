use super::references::Reference;

#[derive(Debug)]
pub struct GitServer
{
    pub version: u32,
    pub capabilities: Vec<String>,
    pub shallow: Vec<String>,
    pub references: Vec<Reference>,
}

