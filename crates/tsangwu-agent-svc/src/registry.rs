use std::collections::HashMap;
use std::sync::Arc;
use super::Agent;

pub struct AgentRegistry {
    agents: HashMap<String, Arc<dyn Agent>>,
}

impl AgentRegistry {
    pub fn new() -> Self {
        Self { agents: HashMap::new() }
    }

    pub fn register(&mut self, agent: Arc<dyn Agent>) {
        self.agents.insert(agent.name().to_string(), agent);
    }

    pub fn get(&self, name: &str) -> Option<Arc<dyn Agent>> {
        self.agents.get(name).cloned()
    }

    pub fn names(&self) -> Vec<String> {
        self.agents.keys().cloned().collect()
    }
}

impl Default for AgentRegistry {
    fn default() -> Self {
        Self::new()
    }
}
