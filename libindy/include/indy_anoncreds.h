#ifndef __anoncreds__included__
#define __anoncreds__included__

#ifdef __cplusplus
extern "C" {
#endif
    
    extern indy_error_t indy_issuer_create_schema(indy_handle_t command_handle,
                                                  const char *  issuer_did,
                                                  const char *  name,
                                                  const char *  version,
                                                  const char *  attr_names,

                                                  void           (*cb)(indy_handle_t xcommand_handle,
                                                                       indy_error_t  err,
                                                                       const char*   id,
                                                                       const char*   schema_json)
                                                  );

    extern indy_error_t indy_issuer_create_and_store_credential_def(indy_handle_t command_handle,
                                                                    indy_handle_t wallet_handle,
                                                                    const char *  issuer_did,
                                                                    const char *  schema_json,
                                                                    const char *  tag,
                                                                    const char *  type_,
                                                                    const char *  config_json,

                                                                    void           (*cb)(indy_handle_t xcommand_handle,
                                                                                         indy_error_t  err,
                                                                                         const char*   cred_def_id,
                                                                                         const char*   cred_def_json)
                                                                    );
    
    extern indy_error_t indy_issuer_create_and_store_revoc_reg(indy_handle_t command_handle,
                                                               indy_handle_t wallet_handle,
                                                               const char *  issuer_did,
                                                               const char *  type_,
                                                               const char *  tag,
                                                               const char *  cred_def_id,
                                                               const char *  config_json,
                                                               const char *  tails_writer_type,
                                                               const char *  tails_writer_config,

                                                               void           (*cb)(indy_handle_t xcommand_handle,
                                                                                    indy_error_t  err,
                                                                                    const char*   revoc_reg_id,
                                                                                    const char*   revoc_reg_def_json,
                                                                                    const char*   revoc_reg_entry_json)
                                                               );

    extern indy_error_t indy_issuer_create_credential_offer(indy_handle_t command_handle,
                                                            indy_handle_t wallet_handle,
                                                            const char *  cred_def_id,

                                                            void           (*cb)(indy_handle_t xcommand_handle,
                                                                                 indy_error_t  err,
                                                                                 const char*   cred_offer_json)
                                                            );
    
    extern indy_error_t indy_issuer_create_credential(indy_handle_t command_handle,
                                                      indy_handle_t wallet_handle,
                                                      const char *  cred_offer_json,
                                                      const char *  cred_req_json,
                                                      const char *  cred_values_json,
                                                      const char *  rev_reg_id,
                                                      indy_i32_t    blob_storage_reader_handle,

                                                      void           (*cb)(indy_handle_t xcommand_handle,
                                                                           indy_error_t  err,
                                                                           const char*   cred_json,
                                                                           const char*   cred_revoc_id,
                                                                           const char*   revoc_reg_delta_json)
                                                      );
    
    extern indy_error_t indy_issuer_revoke_credential(indy_handle_t command_handle,
                                                      indy_handle_t wallet_handle,
                                                      indy_i32_t    blob_storage_reader_handle,
                                                      const char *  rev_reg_id,
                                                      const char *  cred_revoc_id,

                                                      void           (*cb)(indy_handle_t xcommand_handle,
                                                                           indy_error_t  err,
                                                                           const char*   revoc_reg_delta_json)
                                                      );

    extern indy_error_t indy_issuer_recover_credential(indy_handle_t command_handle,
                                                       indy_handle_t wallet_handle,
                                                       indy_i32_t    blob_storage_reader_handle,
                                                       const char *  rev_reg_id,
                                                       const char *  cred_revoc_id,

                                                       void           (*cb)(indy_handle_t xcommand_handle,
                                                                            indy_error_t  err,
                                                                            const char*   revoc_reg_delta_json)
                                                       );


    extern indy_error_t indy_issuer_merge_revocation_registry_deltas(indy_handle_t command_handle,
                                                                     const char *  rev_reg_delta_json,
                                                                     const char *  other_rev_reg_delta_json,

                                                                     void           (*cb)(indy_handle_t xcommand_handle,
                                                                                          indy_error_t  err,
                                                                                          const char*   merged_rev_reg_delta)
                                                                     );

    extern indy_error_t indy_prover_create_master_secret(indy_handle_t command_handle,
                                                         indy_handle_t wallet_handle,
                                                         const char *  master_secret_id,

                                                         void           (*cb)(indy_handle_t xcommand_handle,
                                                                              indy_error_t  err,
                                                                              const char*   out_master_secret_id)
                                                         );
    
    
    extern indy_error_t indy_prover_create_credential_req(indy_handle_t command_handle,
                                                          indy_handle_t wallet_handle,
                                                          const char *  prover_did,
                                                          const char *  cred_offer_json,
                                                          const char *  cred_def_json,
                                                          const char *  master_secret_id,

                                                          void           (*cb)(indy_handle_t xcommand_handle,
                                                                               indy_error_t  err,
                                                                               const char*   cred_req_json,
                                                                               const char*   cred_req_metadata_json)
                                                          );

    extern indy_error_t indy_prover_store_credential(indy_handle_t command_handle,
                                                     indy_handle_t wallet_handle,
                                                     const char *  cred_id,
                                                     const char *  cred_req_json,
                                                     const char *  cred_req_metadata_json,
                                                     const char *  cred_json,
                                                     const char *  cred_def_json,
                                                     const char *  rev_reg_def_json,

                                                     void           (*cb)(indy_handle_t xcommand_handle,
                                                                          indy_error_t  err,
                                                                          const char*   out_cred_id)
                                                     );
    
    extern indy_error_t indy_prover_get_credentials(indy_handle_t command_handle,
                                                    indy_handle_t wallet_handle,
                                                    const char *  filter_json,

                                                    void           (*cb)(indy_handle_t xcommand_handle,
                                                                         indy_error_t  err,
                                                                         const char*   credentials_json)
                                                    );
    
    
    extern indy_error_t indy_prover_get_credentials_for_proof_req(indy_handle_t command_handle,
                                                                  indy_handle_t wallet_handle,
                                                                  const char *  proof_request_json,

                                                                  void           (*cb)(indy_handle_t xcommand_handle,
                                                                                       indy_error_t  err,
                                                                                       const char*   credentials_json)
                                                                  );
    
    
    extern indy_error_t indy_prover_create_proof(indy_handle_t command_handle,
                                                 indy_handle_t wallet_handle,
                                                 const char *  proof_req_json,
                                                 const char *  requested_credentials_json,
                                                 const char *  master_secret_name,
                                                 const char *  schemas_json,
                                                 const char *  credential_defs_json,
                                                 const char *  rev_states_json,

                                                 void           (*cb)(indy_handle_t xcommand_handle,
                                                                      indy_error_t  err,
                                                                      const char*   proof_json)
                                                 );


    extern indy_error_t indy_verifier_verify_proof(indy_handle_t command_handle,
                                                   const char *  proof_request_json,
                                                   const char *  proof_json,
                                                   const char *  schemas_json,
                                                   const char *  credential_defs_jsons,
                                                   const char *  rev_reg_defs_json,
                                                   const char *  rev_regs_json,

                                                   void           (*cb)(indy_handle_t xcommand_handle,
                                                                        indy_error_t  err,
                                                                        indy_bool_t   valid )
                                                   );


    extern indy_error_t indy_create_revocation_state(indy_handle_t command_handle,
                                                     indy_i32_t    blob_storage_reader_handle,
                                                     const char *  rev_reg_def_json,
                                                     const char *  rev_reg_delta_json,
                                                     indy_u64_t    timestamp,
                                                     const char *  cred_rev_id,

                                                     void           (*cb)(indy_handle_t xcommand_handle,
                                                                          indy_error_t  err,
                                                                          const char*   rev_state_json)
                                                     );


    extern indy_error_t indy_update_revocation_state(indy_handle_t command_handle,
                                                     indy_i32_t    blob_storage_reader_handle,
                                                     const char *  rev_state_json,
                                                     const char *  rev_reg_def_json,
                                                     const char *  rev_reg_delta_json,
                                                     indy_u64_t    timestamp,
                                                     const char *  cred_rev_id,

                                                     void           (*cb)(indy_handle_t xcommand_handle,
                                                                          indy_error_t  err,
                                                                          const char*   updated_rev_state_json)
                                                     );

#ifdef __cplusplus
}
#endif

#endif
