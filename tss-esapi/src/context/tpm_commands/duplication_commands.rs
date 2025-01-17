// Copyright 2021 Contributors to the Parsec project.
// SPDX-License-Identifier: Apache-2.0
use crate::Context;
use crate::{
    handles::ObjectHandle,
    structures::{Data, EncryptedSecret, Private, SymmetricDefinitionObject},
    tss2_esys::*,
    Error, Result,
};
use log::error;

use std::convert::TryFrom;
use std::ptr::null_mut;

impl Context {
    /// Duplicate a loaded object so that it may be used in a different hierarchy.
    ///
    /// # Details
    /// This command duplicates a loaded object so that it may be used in a different hierarchy.
    /// The new parent key for the duplicate may be on the same or different TPM or the Null hierarchy.
    /// Only the public area of `new_parent_handle` is required to be loaded.
    ///
    /// # Arguments
    /// * `object_to_duplicate` - An [ObjectHandle] of the object that will be duplicated.
    /// * `new_parent_handle` - An [ObjectHandle] of the new parent.
    /// * `encryption_key_in` - An optional encryption key.
    /// * `symmetric_alg` - Symmetric algorithm to be used for the inner wrapper.
    ///
    /// The `object_to_duplicate` need to be have Fixed TPM and Fixed Parent attributes set to `false`.
    ///
    /// # Returns
    /// The command returns a tuple consisting of:
    /// * `encryption_key_out` - TPM generated, symmetric encryption key for the inner wrapper if
    ///   `symmetric_alg` is not `Null`.
    /// * `duplicate` - Private area that may be encrypted.
    /// * `out_sym_seed` - Seed protected by the asymmetric algorithms of new parent.
    ///
    /// ```rust
    /// # use std::convert::{TryFrom, TryInto};
    /// # use tss_esapi::attributes::{ObjectAttributesBuilder, SessionAttributesBuilder};
    /// # use tss_esapi::constants::{tss::TPM2_CC_Duplicate, SessionType};
    /// # use tss_esapi::handles::ObjectHandle;
    /// # use tss_esapi::interface_types::{
    /// #     algorithm::{HashingAlgorithm, PublicAlgorithm},
    /// #     key_bits::RsaKeyBits,
    /// #     resource_handles::Hierarchy,
    /// #     session_handles::PolicySession,
    /// # };
    /// # use tss_esapi::structures::SymmetricDefinition;
    /// # use tss_esapi::structures::{
    /// #     PublicBuilder, PublicKeyRsa, PublicRsaParametersBuilder, RsaScheme,
    /// #     RsaExponent,
    /// # };
    /// use tss_esapi::structures::SymmetricDefinitionObject;
    /// # use tss_esapi::abstraction::cipher::Cipher;
    /// # use tss_esapi::{Context, TctiNameConf};
    /// #
    /// # let mut context = // ...
    /// #     Context::new(
    /// #         TctiNameConf::from_environment_variable().expect("Failed to get TCTI"),
    /// #     ).expect("Failed to create Context");
    /// #
    /// # let trial_session = context
    /// #     .start_auth_session(
    /// #         None,
    /// #         None,
    /// #         None,
    /// #         SessionType::Trial,
    /// #         SymmetricDefinition::AES_256_CFB,
    /// #         HashingAlgorithm::Sha256,
    /// #     )
    /// #     .expect("Start auth session failed")
    /// #     .expect("Start auth session returned a NONE handle");
    /// #
    /// # let (policy_auth_session_attributes, policy_auth_session_attributes_mask) =
    /// #     SessionAttributesBuilder::new()
    /// #         .with_decrypt(true)
    /// #         .with_encrypt(true)
    /// #         .build();
    /// # context
    /// #     .tr_sess_set_attributes(
    /// #         trial_session,
    /// #         policy_auth_session_attributes,
    /// #         policy_auth_session_attributes_mask,
    /// #     )
    /// #     .expect("tr_sess_set_attributes call failed");
    /// #
    /// # let policy_session = PolicySession::try_from(trial_session)
    /// #     .expect("Failed to convert auth session into policy session");
    /// #
    /// # context
    /// #     .policy_auth_value(policy_session)
    /// #     .expect("Policy auth value");
    /// #
    /// # context
    /// #     .policy_command_code(policy_session, TPM2_CC_Duplicate)
    /// #     .expect("Policy command code");
    /// #
    /// # /// Digest of the policy that allows duplication
    /// # let digest = context
    /// #     .policy_get_digest(policy_session)
    /// #     .expect("Could retrieve digest");
    /// #
    /// # drop(context);
    /// # let mut context = // ...
    /// #     Context::new(
    /// #         TctiNameConf::from_environment_variable().expect("Failed to get TCTI"),
    /// #     ).expect("Failed to create Context");
    /// #
    /// # let session = context
    /// #     .start_auth_session(
    /// #         None,
    /// #         None,
    /// #         None,
    /// #         SessionType::Hmac,
    /// #         SymmetricDefinition::AES_256_CFB,
    /// #         HashingAlgorithm::Sha256,
    /// #     )
    /// #     .expect("Start auth session failed")
    /// #     .expect("Start auth session returned a NONE handle");
    /// #
    /// # let (session_attributes, session_attributes_mask) = SessionAttributesBuilder::new()
    /// #     .with_decrypt(true)
    /// #     .with_encrypt(true)
    /// #     .build();
    /// #
    /// # context.tr_sess_set_attributes(
    /// #     session,
    /// #     session_attributes,
    /// #     session_attributes_mask,
    /// # ).unwrap();
    /// #
    /// # context.set_sessions((Some(session), None, None));
    /// #
    /// # // Attributes of parent objects. The `restricted` attribute need
    /// # // to be `true` so that parents can act as storage keys.
    /// # let parent_object_attributes = ObjectAttributesBuilder::new()
    /// #     .with_fixed_tpm(true)
    /// #     .with_fixed_parent(true)
    /// #     .with_sensitive_data_origin(true)
    /// #     .with_user_with_auth(true)
    /// #     .with_decrypt(true)
    /// #     .with_sign_encrypt(false)
    /// #     .with_restricted(true)
    /// #     .build()
    /// #     .unwrap();
    /// #
    /// # let parent_public = PublicBuilder::new()
    /// #     .with_public_algorithm(PublicAlgorithm::Rsa)
    /// #     .with_name_hashing_algorithm(HashingAlgorithm::Sha256)
    /// #     .with_object_attributes(parent_object_attributes)
    /// #     .with_rsa_parameters(
    /// #         PublicRsaParametersBuilder::new_restricted_decryption_key(
    /// #             Cipher::aes_256_cfb().try_into().unwrap(),
    /// #             RsaKeyBits::Rsa2048,
    /// #             RsaExponent::default(),
    /// #         )
    /// #         .build()
    /// #         .unwrap(),
    /// #     )
    /// #     .with_rsa_unique_identifier(&PublicKeyRsa::default())
    /// #     .build()
    /// #     .unwrap();
    /// #
    /// # let parent_of_object_to_duplicate_handle = context
    /// #     .create_primary(
    /// #         Hierarchy::Owner,
    /// #         &parent_public,
    /// #         None,
    /// #         None,
    /// #         None,
    /// #         None,
    /// #     )
    /// #     .unwrap()
    /// #     .key_handle;
    /// #
    /// # // Fixed TPM and Fixed Parent should be "false" for an object
    /// # // to be elligible for duplication
    /// # let object_attributes = ObjectAttributesBuilder::new()
    /// #     .with_fixed_tpm(false)
    /// #     .with_fixed_parent(false)
    /// #     .with_sensitive_data_origin(true)
    /// #     .with_user_with_auth(true)
    /// #     .with_decrypt(true)
    /// #     .with_sign_encrypt(true)
    /// #     .with_restricted(false)
    /// #     .build()
    /// #     .expect("Attributes to be valid");
    /// #
    /// # let public_child = PublicBuilder::new()
    /// #     .with_public_algorithm(PublicAlgorithm::Rsa)
    /// #     .with_name_hashing_algorithm(HashingAlgorithm::Sha256)
    /// #     .with_object_attributes(object_attributes)
    /// #     .with_auth_policy(&digest)
    /// #     .with_rsa_parameters(
    /// #         PublicRsaParametersBuilder::new()
    /// #             .with_scheme(RsaScheme::Null)
    /// #             .with_key_bits(RsaKeyBits::Rsa2048)
    /// #             .with_is_signing_key(true)
    /// #             .with_is_decryption_key(true)
    /// #             .with_restricted(false)
    /// #             .build()
    /// #             .expect("Params to be valid"),
    /// #     )
    /// #     .with_rsa_unique_identifier(&PublicKeyRsa::default())
    /// #     .build()
    /// #     .expect("public to be valid");
    /// #
    /// # let result = context
    /// #     .create(
    /// #         parent_of_object_to_duplicate_handle,
    /// #         &public_child,
    /// #         None,
    /// #         None,
    /// #         None,
    /// #         None,
    /// #     )
    /// #     .unwrap();
    /// #
    /// # let object_to_duplicate_handle: ObjectHandle = context
    /// #     .load(
    /// #         parent_of_object_to_duplicate_handle,
    /// #         result.out_private.clone(),
    /// #         &result.out_public,
    /// #     )
    /// #     .unwrap()
    /// #     .into();
    /// #
    /// # let new_parent_handle: ObjectHandle = context
    /// #     .create_primary(
    /// #         Hierarchy::Owner,
    /// #         &parent_public,
    /// #         None,
    /// #         None,
    /// #         None,
    /// #         None,
    /// #     )
    /// #     .unwrap()
    /// #     .key_handle
    /// #     .into();
    /// #
    /// # context.set_sessions((None, None, None));
    /// #
    /// # // Create a Policy session with the same exact attributes
    /// # // as the trial session so that the session digest stays
    /// # // the same.
    /// # let policy_auth_session = context
    /// #     .start_auth_session(
    /// #         None,
    /// #         None,
    /// #         None,
    /// #         SessionType::Policy,
    /// #         SymmetricDefinition::AES_256_CFB,
    /// #         HashingAlgorithm::Sha256,
    /// #     )
    /// #     .expect("Start auth session failed")
    /// #     .expect("Start auth session returned a NONE handle");
    /// #
    /// # let (policy_auth_session_attributes, policy_auth_session_attributes_mask) =
    /// #     SessionAttributesBuilder::new()
    /// #         .with_decrypt(true)
    /// #         .with_encrypt(true)
    /// #         .build();
    /// # context
    /// #     .tr_sess_set_attributes(
    /// #         policy_auth_session,
    /// #         policy_auth_session_attributes,
    /// #         policy_auth_session_attributes_mask,
    /// #     )
    /// #     .expect("tr_sess_set_attributes call failed");
    /// #
    /// # let policy_session = PolicySession::try_from(policy_auth_session)
    /// #     .expect("Failed to convert auth session into policy session");
    /// #
    /// # context
    /// #     .policy_auth_value(policy_session)
    /// #     .expect("Policy auth value");
    /// #
    /// # context
    /// #     .policy_command_code(policy_session, TPM2_CC_Duplicate)
    /// #     .unwrap();
    /// #
    /// # context.set_sessions((Some(policy_auth_session), None, None));
    ///
    /// let (encryption_key_out, duplicate, out_sym_seed) = context
    ///     .duplicate(
    ///         object_to_duplicate_handle,
    ///         new_parent_handle,
    ///         None,
    ///         SymmetricDefinitionObject::Null,
    ///     )
    ///     .unwrap();
    /// # eprintln!("D: {:?}, P: {:?}, S: {:?}", encryption_key_out, duplicate, out_sym_seed);
    /// ```
    pub fn duplicate(
        &mut self,
        object_to_duplicate: ObjectHandle,
        new_parent_handle: ObjectHandle,
        encryption_key_in: Option<Data>,
        symmetric_alg: SymmetricDefinitionObject,
    ) -> Result<(Data, Private, EncryptedSecret)> {
        let mut encryption_key_out = null_mut();
        let mut duplicate = null_mut();
        let mut out_sym_seed = null_mut();
        let ret = unsafe {
            Esys_Duplicate(
                self.mut_context(),
                object_to_duplicate.into(),
                new_parent_handle.into(),
                self.required_session_1()?,
                self.optional_session_2(),
                self.optional_session_3(),
                &encryption_key_in.unwrap_or_default().into(),
                &symmetric_alg.into(),
                &mut encryption_key_out,
                &mut duplicate,
                &mut out_sym_seed,
            )
        };
        let ret = Error::from_tss_rc(ret);

        if ret.is_success() {
            let encryption_key_out = unsafe { Data::try_from(*encryption_key_out)? };
            let duplicate = unsafe { Private::try_from(*duplicate)? };
            let out_sym_seed = unsafe { EncryptedSecret::try_from(*out_sym_seed)? };
            Ok((encryption_key_out, duplicate, out_sym_seed))
        } else {
            error!("Error when performing duplication: {}", ret);
            Err(ret)
        }
    }

    // Missing function: Rewrap
    // Missing function: Import
}
