use std::fmt::{Display, Formatter};
use super::{Recipe, CookResult, RecipeError};

pub struct GroupRecipe {
    name: String,
    recipes: Vec<Box<dyn Recipe>>,
}

impl GroupRecipe {
    pub fn new(name: String, recipes: Vec<Box<dyn Recipe>>) -> Self {
        Self { name, recipes }
    }
}

impl Recipe for GroupRecipe {
    fn name(&self) -> &str { &self.name }
    fn description(&self) -> Option<&str> { None }
    
    fn cook(&self) -> Result<CookResult, RecipeError> {
        let mut results = Vec::new();
        for recipe in &self.recipes {
            println!("[{}] Building...", recipe.name());
            results.push(recipe.cook()?);
            println!("[{}] Done\n", recipe.name());
        }
        Ok(CookResult::Batch(results))
    }
    
    fn validate(&self) -> Result<(), RecipeError> {
        for recipe in &self.recipes {
            recipe.validate()?;
        }
        Ok(())
    }
}

impl Display for GroupRecipe {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let names: Vec<_> = self.recipes.iter().map(|r| r.name()).collect();
        write!(f, "Group: {} -> [{}]", self.name, names.join(", "))
    }
}