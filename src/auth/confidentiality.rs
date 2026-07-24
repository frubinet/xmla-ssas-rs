// SPDX-License-Identifier: MPL-2.0

use anyhow::bail;
use sspi::{EncryptionFlags, Ntlm, SecurityBufferRef, Sspi};

pub(crate) fn encrypt_ssas_message(ntlm: &mut Ntlm, plaintext: &[u8]) -> anyhow::Result<Vec<u8>> {
    // TODO: support multiple blocks
    let data_length = u16::try_from(plaintext.len())?;

    let sizes = ntlm.query_context_sizes()?;
    let token_length = usize::try_from(sizes.security_trailer)?;
    let token_length_u16 = u16::try_from(token_length)?;

    let mut data = plaintext.to_vec();
    let mut token = vec![0_u8; token_length];

    {
        let mut buffers = [
            SecurityBufferRef::token_buf(&mut token),
            SecurityBufferRef::data_buf(&mut data),
        ];

        ntlm.encrypt_message(EncryptionFlags::empty(), &mut buffers)?;
    }

    let mut block = Vec::with_capacity(4 + data.len() + token.len());
    block.extend_from_slice(&data_length.to_le_bytes());
    block.extend_from_slice(&token_length_u16.to_le_bytes());
    block.extend_from_slice(&data);
    block.extend_from_slice(&token);

    Ok(block)
}

pub(crate) fn decrypt_ssas_message(ntlm: &mut Ntlm, encrypted_message: &[u8]) -> anyhow::Result<Vec<u8>> {
    let mut remaining = encrypted_message;
    let mut plaintext = Vec::new();

    while !remaining.is_empty() {
        // DATA_SIZE + TOKEN_SIZE
        if remaining.len() < 4 {
            bail!("truncated SSAS encryption-block header");
        }

        let data_length = usize::from(u16::from_le_bytes([remaining[0], remaining[1]]));

        let token_length = usize::from(u16::from_le_bytes([remaining[2], remaining[3]]));

        let block_length = 4 + data_length + token_length;

        if remaining.len() < block_length {
            bail!(
                "truncated SSAS encryption block: expected {block_length} bytes, found {}",
                remaining.len()
            );
        }

        let data_start = 4;
        let data_end = data_start + data_length;
        let token_end = data_end + token_length;

        let mut data = remaining[data_start..data_end].to_vec();
        let mut token = remaining[data_end..token_end].to_vec();

        {
            let mut buffers = [
                SecurityBufferRef::token_buf(&mut token),
                SecurityBufferRef::data_buf(&mut data),
            ];

            // Decrypts DATA in place and verifies its signature.
            ntlm.decrypt_message(&mut buffers)?;
        }

        plaintext.extend_from_slice(&data);
        remaining = &remaining[token_end..];
    }

    Ok(plaintext)
}
