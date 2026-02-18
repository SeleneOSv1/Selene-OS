#![forbid(unsafe_code)]

use std::marker::PhantomData;

use selene_kernel_contracts::ph1k::PH1K_IMPLEMENTATION_ID;
use selene_kernel_contracts::ContractViolation;

pub const PH1_K_ENGINE_ID: &str = "PH1.K";
pub const PH1_K_ACTIVE_IMPLEMENTATION_IDS: &[&str] = &[PH1K_IMPLEMENTATION_ID];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ph1kWiringConfig {
    pub ph1k_enabled: bool,
}

impl Ph1kWiringConfig {
    pub fn mvp_v1(ph1k_enabled: bool) -> Self {
        Self { ph1k_enabled }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ph1kWiringOutcome<O> {
    NotInvokedDisabled,
    Forwarded(Vec<O>),
}

pub trait Ph1kEngine<Event, Output> {
    fn handle_for_implementation(
        &mut self,
        implementation_id: &str,
        event: Event,
    ) -> Result<Vec<Output>, ContractViolation>;
}

#[derive(Debug, Clone)]
pub struct Ph1kWiring<E, Event, Output>
where
    E: Ph1kEngine<Event, Output>,
{
    config: Ph1kWiringConfig,
    engine: E,
    _event: PhantomData<Event>,
    _output: PhantomData<Output>,
}

impl<E, Event, Output> Ph1kWiring<E, Event, Output>
where
    E: Ph1kEngine<Event, Output>,
{
    pub fn new(config: Ph1kWiringConfig, engine: E) -> Self {
        Self {
            config,
            engine,
            _event: PhantomData,
            _output: PhantomData,
        }
    }

    pub fn run_event(
        &mut self,
        event: Event,
    ) -> Result<Ph1kWiringOutcome<Output>, ContractViolation> {
        if !self.config.ph1k_enabled {
            return Ok(Ph1kWiringOutcome::NotInvokedDisabled);
        }

        let out = self
            .engine
            .handle_for_implementation(PH1K_IMPLEMENTATION_ID, event)?;
        Ok(Ph1kWiringOutcome::Forwarded(out))
    }

    #[allow(dead_code)]
    pub fn engine_ref(&self) -> &E {
        &self.engine
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use selene_kernel_contracts::ContractViolation;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum FakeEvent {
        Tick,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum FakeOutput {
        EventForwarded,
    }

    #[derive(Debug, Default, Clone)]
    struct FakeEngine {
        calls: usize,
        force_unknown_impl_failure: bool,
    }

    impl Ph1kEngine<FakeEvent, FakeOutput> for FakeEngine {
        fn handle_for_implementation(
            &mut self,
            implementation_id: &str,
            _event: FakeEvent,
        ) -> Result<Vec<FakeOutput>, ContractViolation> {
            self.calls = self.calls.saturating_add(1);
            if self.force_unknown_impl_failure || implementation_id != PH1K_IMPLEMENTATION_ID {
                return Err(ContractViolation::InvalidValue {
                    field: "ph1_k.implementation_id",
                    reason: "unknown implementation_id",
                });
            }
            Ok(vec![FakeOutput::EventForwarded])
        }
    }

    #[test]
    fn at_k_wiring_01_disabled_does_not_invoke_engine() {
        let engine = FakeEngine::default();
        let mut wiring = Ph1kWiring::new(Ph1kWiringConfig::mvp_v1(false), engine);
        let out = wiring.run_event(FakeEvent::Tick).unwrap();
        assert_eq!(out, Ph1kWiringOutcome::NotInvokedDisabled);
        assert_eq!(wiring.engine_ref().calls, 0);
    }

    #[test]
    fn at_k_wiring_02_enabled_forwards_event() {
        let engine = FakeEngine::default();
        let mut wiring = Ph1kWiring::new(Ph1kWiringConfig::mvp_v1(true), engine);
        let out = wiring.run_event(FakeEvent::Tick).unwrap();
        assert_eq!(
            out,
            Ph1kWiringOutcome::Forwarded(vec![FakeOutput::EventForwarded])
        );
        assert_eq!(wiring.engine_ref().calls, 1);
    }

    #[test]
    fn at_k_wiring_03_fail_closed_on_unknown_implementation_error() {
        let engine = FakeEngine {
            calls: 0,
            force_unknown_impl_failure: true,
        };
        let mut wiring = Ph1kWiring::new(Ph1kWiringConfig::mvp_v1(true), engine);
        let out = wiring.run_event(FakeEvent::Tick);
        assert!(matches!(
            out,
            Err(ContractViolation::InvalidValue {
                field: "ph1_k.implementation_id",
                reason: "unknown implementation_id",
            })
        ));
    }

    #[test]
    fn at_k_wiring_04_active_implementation_list_is_locked() {
        assert_eq!(PH1_K_ENGINE_ID, "PH1.K");
        assert_eq!(PH1_K_ACTIVE_IMPLEMENTATION_IDS, &["PH1.K.001"]);
    }
}
