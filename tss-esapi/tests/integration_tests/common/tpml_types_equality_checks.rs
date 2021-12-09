use tss_esapi::tss2_esys::{TPML_PCR_SELECTION, TPML_TAGGED_TPM_PROPERTY};

macro_rules! ensure_list_equality {
    ($name:ident, $list_type:ident, $items_field_name:ident, $item_equality_func:ident) => {
        #[allow(dead_code)]
        pub fn $name(expected: &$list_type, actual: &$list_type) {
            assert_eq!(
                expected.count,
                actual.count,
                "'count' value in {}, mismatch between actual and expected",
                stringify!($list_type)
            );
            expected.$items_field_name[..expected.count as usize]
                .iter()
                .zip(actual.$items_field_name[..actual.count as usize].iter())
                .for_each(|(expected, actual)| {
                    crate::common::$item_equality_func(expected, actual)
                });
        }
    };
}

ensure_list_equality!(
    ensure_tpml_pcr_selection_equality,
    TPML_PCR_SELECTION,
    pcrSelections,
    ensure_tpms_pcr_selection_equality
);

ensure_list_equality!(
    ensure_tpml_tagged_tpm_property_equality,
    TPML_TAGGED_TPM_PROPERTY,
    tpmProperty,
    ensure_tpms_tagged_property_equality
);
