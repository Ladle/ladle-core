use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct LadleTOML {
    lang: Language,
    spec: Specification,
    tests: Vec<Test>
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct Language {

}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct Specification {

}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct Test {
    
}
