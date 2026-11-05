// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
use cad_craft::domain::*;
use serde_json::json;

#[test]
fn physical_ids_use_v7_and_preserve_identity() {
    macro_rules! check {
        ($($ty:ty),+) => {$(
            let id = <$ty>::new();
            assert_eq!(id.as_uuid().get_version_num(), 7);
            assert_eq!(id.to_string().parse::<$ty>().unwrap(), id);
            assert_eq!(serde_json::from_value::<$ty>(json!(id)).unwrap(), id);
            assert!(serde_json::from_value::<$ty>(json!("00000000-0000-0000-0000-000000000000")).is_err());
        )+};
    }
    check!(ConductorId, CableId, CableCoreId, TerminalStripId);
}

#[test]
fn physical_quantities_enforce_units_finiteness_and_bounds() {
    for invalid in [0.0, -1.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert!(CrossSectionMm2::new(invalid).is_err());
    }
    for invalid in [-1.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert!(LengthMm::new(invalid).is_err());
    }
    assert_eq!(CrossSectionMm2::new(1.5).unwrap().get(), 1.5);
    assert_eq!(LengthMm::new(0.0).unwrap().get(), 0.0);
    assert_eq!(LengthMm::from_meters(2.5).unwrap().get(), 2500.0);
    assert!(LengthMm::from_meters(f64::MAX).is_err());
    assert!(serde_json::from_value::<CrossSectionMm2>(json!(0)).is_err());
    assert!(serde_json::from_value::<LengthMm>(json!(-1)).is_err());
    assert!(serde_json::from_value::<ConductorColor>(json!("invalid")).is_err());
    assert!(serde_json::from_value::<ConductorMaterial>(json!("invalid")).is_err());
    assert!(serde_json::from_value::<ConductorFunction>(json!("invalid")).is_err());
    assert!(serde_json::from_value::<CableKind>(json!("invalid")).is_err());
}

#[test]
fn marking_requires_color_or_valid_identification_without_inferring_function() {
    assert!(ConductorMarking::new(None, None).is_err());
    let m = ConductorMarking::new(
        Some(ConductorColor::GreenYellow),
        Some(DesignationPart::new("12A").unwrap()),
    )
    .unwrap();
    assert_eq!(
        serde_json::from_value::<ConductorMarking>(json!(m)).unwrap(),
        m
    );
    assert!(
        serde_json::from_value::<ConductorMarking>(json!({"color":null,"identification":null}))
            .is_err()
    );
    assert!(
        serde_json::from_value::<ConductorMarking>(json!({"color":null,"identification":" "}))
            .is_err()
    );
    let a = PhysicalTermination::new(TerminalId::new(), DesignationPart::new("A").unwrap());
    let b = PhysicalTermination::new(TerminalId::new(), DesignationPart::new("B").unwrap());
    let c = Conductor::new(
        ProjectId::new(),
        DesignationPart::new("W1").unwrap(),
        a,
        b,
        CrossSectionMm2::new(1.5).unwrap(),
        m,
    )
    .unwrap();
    assert_eq!(c.function(), ConductorFunction::Unspecified);
    assert_eq!(c.length(), None);
    assert_eq!(c.material(), None);
}

#[test]
fn conductor_rejects_same_physical_position_but_allows_two_positions_of_one_terminal() {
    let terminal = TerminalId::new();
    let a = PhysicalTermination::new(terminal, DesignationPart::new("A").unwrap());
    let b = PhysicalTermination::new(terminal, DesignationPart::new("B").unwrap());
    let make = |target| {
        Conductor::new(
            ProjectId::new(),
            DesignationPart::new("W1").unwrap(),
            a.clone(),
            target,
            CrossSectionMm2::new(1.5).unwrap(),
            ConductorMarking::color(ConductorColor::Blue),
        )
    };
    assert!(make(a.clone()).is_err());
    let c = make(b).unwrap();
    let mut data = json!(c);
    data["target"] = data["source"].clone();
    assert!(serde_json::from_value::<Conductor>(data).is_err());
}

#[test]
fn terminal_positions_reject_duplicates_and_old_terminals_have_no_positions() {
    let t = Terminal::new(
        ProjectId::new(),
        TerminalOwner::Device(DeviceId::new()),
        DesignationPart::new("1").unwrap(),
    );
    let a = DesignationPart::new("A").unwrap();
    assert!(
        t.clone()
            .with_positions(vec![a.clone(), a.clone()])
            .is_err()
    );
    let mut data = json!(t.with_positions(vec![a.clone()]).unwrap());
    data["positions"] = json!([a, a]);
    assert!(serde_json::from_value::<Terminal>(data.clone()).is_err());
    data.as_object_mut().unwrap().remove("positions");
    assert_eq!(
        serde_json::from_value::<Terminal>(data)
            .unwrap()
            .positions()
            .count(),
        0
    );
}

#[test]
fn cable_capacity_is_positive_and_validated_in_serde() {
    assert!(
        Cable::new(
            ProjectId::new(),
            DesignationPart::new("C1").unwrap(),
            CableKind::Control,
            0
        )
        .is_err()
    );
    let cable = Cable::new(
        ProjectId::new(),
        DesignationPart::new("C1").unwrap(),
        CableKind::Control,
        4,
    )
    .unwrap();
    let mut data = json!(cable);
    data["nominal_core_count"] = json!(0);
    assert!(serde_json::from_value::<Cable>(data).is_err());
}
