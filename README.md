# stochos
A stochastic music generation engine.

## Supported stochastic sources:
- [ ] [Bernoulli trial](#bernoulli-trial)

<a id="bernoulli-trial"></a>
### Bernoulli trial
A trial with two possible outcomes that can be defined with the $p + q = 1$ relation, where $p$ is the probability of success, and $q$ of failure.
#### Applications
- Describes whether the current event spans a note or rest
#### Parameters
- $p$: probability of success
- $q$: probability of failure
- If $p$ is defined, $q$ cannot be, and vice-versa
