pub mod errors;

#[allow(dead_code)]
mod interpact {
    use std::time;
    use std::sync;
    use errors;

    #[derive(Debug, Clone, Copy)]
    pub enum State {
        Closed,
        Open,
        HalfOpen,
    }

    #[derive(Debug)]
    pub struct Counts {
        requests: u32,
        total_successes: u32,
        total_failures: u32,
        consecutive_successes: u32,
        consecutive_failures: u32,
    }

    impl Counts {
        fn new() -> Counts {
            Counts {
                requests: 0,
                total_successes: 0,
                total_failures: 0,
                consecutive_failures: 0,
                consecutive_successes: 0,
            }
        }

        fn requested(&mut self) {
            self.requests += 1;
        }

        fn failed(&mut self) {
            self.total_failures += 1;
            self.consecutive_failures += 1;
            self.consecutive_successes = 0;
        }

        fn succeeded(&mut self) {
            self.total_successes += 1;
            self.consecutive_successes += 1;
            self.consecutive_failures = 0;
        }

        fn clear(&mut self) {
            self.requests = 0;
            self.total_failures = 0;
            self.total_successes = 0;
            self.consecutive_failures = 0;
            self.consecutive_successes = 0;
        }
    }

    fn default_ready_to_trip(counts: Counts) -> bool {
        counts.consecutive_failures > 5
    }

    pub struct Options<'a> {
        pub name: &'a str,
        pub max_requests: u32,
        pub success_threshold: Option<u32>,
        pub interval: time::Duration,
        pub timeout: time::Duration,
        pub ready_to_trip: fn(counts: Counts) -> bool,
        pub on_state_change: fn(name: String, from: State, to: State),
    }

    pub struct CircuitBreaker {
        name: String,
        max_requests: u32,
        success_threshold: u32,
        interval: time::Duration,
        timeout: time::Duration,
        ready_to_trip: fn(counts: Counts) -> bool,
        on_state_change: fn(name: String, from: State, to: State),
        state: sync::Mutex<State>,
        // generation: u64,
        counts: Counts,
        expires: Option<time::Instant>,
    }

    impl CircuitBreaker {
        pub fn new(o: Options) -> CircuitBreaker {
            let cb_name = String::from(o.name);
            let mr = if o.max_requests == 0 {
                o.max_requests
            } else {
                1
            };

            CircuitBreaker {
                name: cb_name,
                max_requests: mr,
                success_threshold: o.success_threshold.unwrap_or(mr),
                interval: o.interval,
                timeout: if o.timeout > time::Duration::from_secs(0) {
                    o.timeout
                } else {
                    time::Duration::from_secs(60)
                },
                ready_to_trip: o.ready_to_trip,
                on_state_change: o.on_state_change,
                state: sync::Mutex::new(State::Closed),
                counts: Counts::new(),
                expires: None,
            }
        }

        fn prepare_state(&mut self) {
            let mut state = self.state.lock().unwrap();
            match *state {
                State::Closed => {}
                State::HalfOpen => {}
                State::Open => {
                    if self.expires
                           .unwrap_or(time::Instant::now())
                           .duration_since(time::Instant::now()) > time::Duration::from_secs(0) {
                        *state = State::HalfOpen;
                    }
                }
            };
        }

        fn succeeded(&self) {
            unimplemented!();
        }

        fn failed(&mut self) {
            {
                let state = self.state.lock().unwrap();
                match *state {
                    State::Closed => {
                        return;
                    }
                    State::HalfOpen => {}
                    State::Open => {
                        return;
                    }
                }
            }
            self.set_state(State::Open);
        }

        fn set_state(&mut self, new_state: State) {
            let mut state = self.state.lock().unwrap();
            let old_state = *state;
        }

        pub fn execute<T, E>(&mut self, task: fn() -> Result<T, E>) -> Result<Result<T, E>, errors::CircuitBreakerError> {
            self.prepare_state();
            {
                let state = self.state.lock().unwrap();
                match *state {
                    State::Closed => {}
                    State::HalfOpen => {
                        if self.counts.requests > self.max_requests {
                            return Err(errors::CircuitBreakerError {
                                kind: errors::CircuitBreakerErrorKind::TooManyRequestsError,
                                message: "Maximum requests limit has reached while the CircuitBreaker is HalfOpen".into(),
                            });
                        }
                    }
                    State::Open => {
                        return Err(errors::CircuitBreakerError {
                            kind: errors::CircuitBreakerErrorKind::StateOpenError,
                            message: "The CircuitBreaker is open".into(),
                        });
                    }
                };
            }
            self.counts.requested();
            let task_result = task();
            match task_result {
                Ok(res) => {
                    self.succeeded();
                    return Ok(Ok(res));
                }
                Err(err) => {
                    self.failed();
                    return Ok(Err(err));
                }
            }
        }
    }
}
