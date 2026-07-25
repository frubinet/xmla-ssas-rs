// SPDX-License-Identifier: MPL-2.0

use sspi::{
    AcquireCredentialsHandleResult, AuthIdentityBuffers, BufferType, ClientRequestFlags,
    DataRepresentation, Ntlm, SecurityBuffer, SecurityStatus, Sspi, SspiImpl,
};

pub(crate) fn ntlm_step(
    ntlm: &mut Ntlm,
    credentials: &mut AcquireCredentialsHandleResult<Option<AuthIdentityBuffers>>,
    input_token: &[u8],
) -> sspi::Result<(Vec<u8>, SecurityStatus)> {
    let mut input = vec![SecurityBuffer::new(input_token.to_vec(), BufferType::Token)];

    let mut output = vec![SecurityBuffer::new(Vec::new(), BufferType::Token)];

    let mut builder = ntlm
        .initialize_security_context()
        .with_credentials_handle(&mut credentials.credentials_handle)
        .with_context_requirements(
            ClientRequestFlags::CONFIDENTIALITY | ClientRequestFlags::ALLOCATE_MEMORY,
        )
        .with_target_data_representation(DataRepresentation::Native)
        .with_input(&mut input)
        .with_output(&mut output);

    let result = ntlm
        .initialize_security_context_impl(&mut builder)?
        .resolve_to_result()?;

    let status = result.status;

    if matches!(
        status,
        SecurityStatus::CompleteNeeded | SecurityStatus::CompleteAndContinue
    ) {
        ntlm.complete_auth_token(&mut output)?;
    }

    Ok((output.swap_remove(0).buffer, status))
}
