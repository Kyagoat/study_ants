pub struct QLearningMath {
    pub alpha: f32,   // Learning rate
    pub gamma: f32,   // Discount factor
    pub epsilon: f32, // Exploration rate
}

impl QLearningMath {
    pub fn new(alpha: f32, gamma: f32, epsilon: f32) -> Self {
        Self {
            alpha,
            gamma,
            epsilon,
        }
    }

    // Delta = Alpha * (Reward + Gamma * MaxNext - Current)
    pub fn compute_delta(&self, current_q: f32, reward: f32, max_next_q: f32) -> f32 {
        self.alpha * (reward + self.gamma * max_next_q - current_q)
    }
}
