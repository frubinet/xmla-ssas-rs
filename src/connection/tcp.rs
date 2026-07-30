// SPDX-License-Identifier: MPL-2.0

use crate::auth::{decrypt_ssas_message, encrypt_ssas_message, ntlm_step};
use crate::connection::error::{Result, XmlaError};
use crate::dime::{DimeMessage, DimeOptions};
use crate::xmla::{
    Authenticate, FromXml, ToSoap, XmlaDiscover, XmlaDiscoverResponse, XmlaExecute,
    XmlaOperationContent, XmlaProperties, XmlaRestrictions,
};
use anyhow::Context;
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use log::{debug, info};
use roxmltree::{Document, Node};
use sspi::{AuthIdentity, CredentialUse, Ntlm, Sspi, Username};
use std::io;
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

const DEFAULT_CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
const DEFAULT_READ_TIMEOUT: Duration = Duration::from_secs(60);

const SSAS_NS: &str = "http://schemas.microsoft.com/analysisservices/2003/ext";

#[derive(Debug)]
pub struct SsasTcpConnectionOptions {
    host: String,
    port: u16,
    connect_timeout: Duration,
    read_timeout: Duration,
}

pub struct NtlmCredentials {
    pub domain: String,
    pub username: String,
    pub password: String,
}

pub struct SsasTcpConnection {
    stream: TcpStream,
    ntlm: Ntlm,
}

impl SsasTcpConnectionOptions {
    pub fn new(host: impl Into<String>, port: u16) -> Self {
        Self {
            host: host.into(),
            port,
            connect_timeout: DEFAULT_CONNECT_TIMEOUT,
            read_timeout: DEFAULT_READ_TIMEOUT,
        }
    }

    pub fn with_connect_timeout(mut self, timeout: Duration) -> Self {
        self.connect_timeout = timeout;
        self
    }

    pub fn with_read_timeout(mut self, timeout: Duration) -> Self {
        self.read_timeout = timeout;
        self
    }
}

impl SsasTcpConnection {
    pub fn connect(
        options: SsasTcpConnectionOptions,
        credentials: NtlmCredentials,
    ) -> Result<Self> {
        let mut stream = Self::tcp_connect(&options)?;
        let ntlm = Self::authenticate(&mut stream, credentials)?;
        Ok(SsasTcpConnection { stream, ntlm })
    }

    pub fn probe(options: SsasTcpConnectionOptions) -> Result<()> {
        let mut stream = Self::tcp_connect(&options)?;
        Self::send_empty_soap_message(&mut stream)?;
        Ok(())
    }

    pub fn discover(
        &mut self,
        request_type: impl Into<String>,
        restrictions: &XmlaRestrictions,
    ) -> Result<XmlaDiscoverResponse> {
        let stream = &mut self.stream;
        let ntlm = &mut self.ntlm;

        let mut discover = XmlaDiscover::new(request_type);
        discover.restrictions = restrictions.clone();
        discover
            .properties
            .add(XmlaProperties::CONTENT, XmlaOperationContent::Data.as_str())?;
        let soap = discover
            .to_soap()
            .map_err(|error| XmlaError::ProtocolError(error.to_string()))?;
        let encrypted_soap = encrypt_ssas_message(ntlm, &soap)
            .map_err(|error| XmlaError::ProtocolError(error.to_string()))?;
        let request = DimeMessage {
            options: Some(DimeOptions {
                is_negotiated: true,
                ..DimeOptions::default()
            }),
            content_type: Some(String::from("text/xml")),
            data: encrypted_soap,
        };
        request.write_to(stream)?;
        let response = DimeMessage::read_from(stream)?;
        let decrypted = decrypt_ssas_message(ntlm, &response.data)
            .map_err(|error| XmlaError::ProtocolError(error.to_string()))?;
        let xml = std::str::from_utf8(&decrypted)?;
        let document = Document::parse(xml)?;
        XmlaDiscoverResponse::from_xml(Self::get_body_child(&document)?)
    }

    pub fn execute(
        &mut self,
        query: impl Into<String>,
        catalog: impl Into<String>,
    ) -> Result<String> {
        let stream = &mut self.stream;
        let ntlm = &mut self.ntlm;

        let mut execute = XmlaExecute::new(query, catalog)?;
        execute
            .properties
            .add(XmlaProperties::CONTENT, XmlaOperationContent::Data.as_str())?;
        let soap = execute
            .to_soap()
            .map_err(|error| XmlaError::ProtocolError(error.to_string()))?;
        let encrypted_soap = encrypt_ssas_message(ntlm, &soap)
            .map_err(|error| XmlaError::ProtocolError(error.to_string()))?;
        let request = DimeMessage {
            options: Some(DimeOptions {
                is_negotiated: true,
                ..DimeOptions::default()
            }),
            content_type: Some(String::from("text/xml")),
            data: encrypted_soap,
        };
        request.write_to(stream)?;
        let response = DimeMessage::read_from(stream)?;
        let decrypted = decrypt_ssas_message(ntlm, &response.data)
            .map_err(|error| XmlaError::ProtocolError(error.to_string()))?;
        let xml = std::str::from_utf8(&decrypted)?;
        Ok(xml.to_string())
    }

    fn tcp_connect(options: &SsasTcpConnectionOptions) -> Result<TcpStream> {
        info!("Connecting to {}:{}...", options.host, options.port);

        let addresses = (options.host.as_str(), options.port).to_socket_addrs()?;

        let mut last_error = None;

        for address in addresses {
            match TcpStream::connect_timeout(&address, options.connect_timeout) {
                Ok(stream) => {
                    info!("Connected");
                    stream.set_read_timeout(Some(options.read_timeout))?;

                    return Ok(stream);
                }
                Err(error) => last_error = Some(error),
            }
        }

        Err(last_error
            .unwrap_or_else(|| {
                io::Error::new(
                    io::ErrorKind::AddrNotAvailable,
                    "host resolved to no addresses",
                )
            })
            .into())
    }

    fn authenticate(stream: &mut TcpStream, credentials: NtlmCredentials) -> Result<Ntlm> {
        let qualified_username = format!(r"{}\{}", credentials.domain, credentials.username);

        let identity = AuthIdentity {
            username: Username::parse(&qualified_username)
                .map_err(|error| XmlaError::InvalidUsername(error.to_string()))?,
            password: credentials.password.into(),
        };

        let mut ntlm = Ntlm::new();

        let mut credentials = ntlm
            .acquire_credentials_handle()
            .with_credential_use(CredentialUse::Outbound)
            .with_auth_data(&identity)
            .execute(&mut ntlm)?;
        let (negotiate_token, _status) = ntlm_step(&mut ntlm, &mut credentials, &[])?;

        let auth_request = Authenticate {
            sspi_handshake: STANDARD.encode(&negotiate_token),
        };
        let soap = auth_request
            .to_soap()
            .map_err(|error| XmlaError::SerializationError(error.to_string()))?;
        let request = DimeMessage {
            options: Some(DimeOptions::default()),
            content_type: Some(String::from("text/xml")),
            data: soap,
        };
        request.write_to(stream)?;
        let response = DimeMessage::read_from(stream)?;

        let xml = std::str::from_utf8(&response.data)?;
        let document = roxmltree::Document::parse(xml)?;

        let handshake = document
            .descendants()
            .find(|node| {
                node.is_element()
                    && node.tag_name().name() == "SspiHandshake"
                    && node.tag_name().namespace() == Some(SSAS_NS)
            })
            .and_then(|node| node.text())
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| {
                XmlaError::ProtocolError("SspiHandshake missing from response".into())
            })?;

        let handshake_base64: String = handshake
            .chars()
            .filter(|character| !character.is_ascii_whitespace())
            .collect();

        let challenge_token = STANDARD
            .decode(handshake_base64.as_bytes())
            .context("Invalid SspiHandshake Base64")
            .map_err(|error| XmlaError::ProtocolError(error.to_string()))?;

        let (authenticate_token, _status) =
            ntlm_step(&mut ntlm, &mut credentials, &challenge_token)?;

        let auth_request = Authenticate {
            sspi_handshake: STANDARD.encode(&authenticate_token),
        };
        let soap = auth_request
            .to_soap()
            .map_err(|error| XmlaError::ProtocolError(error.to_string()))?;
        let request = DimeMessage {
            options: Some(DimeOptions {
                is_negotiated: true,
                ..DimeOptions::default()
            }),
            content_type: Some(String::from("text/xml")),
            data: soap,
        };
        request.write_to(stream)?;
        let response = DimeMessage::read_from(stream)?;
        if let Some(options) = response.options {
            debug!(
                "SSAS options: compressed: {}, binary_xml:{}",
                options.is_response_compressed, options.is_response_xml_binary
            );
        } else {
            debug!("SSAS response contains no DIME options");
        }

        let xml = std::str::from_utf8(&response.data)?;
        let document = roxmltree::Document::parse(xml)?;
        let handshake_node = document
            .descendants()
            .find(|node| {
                node.is_element()
                    && node.tag_name().name() == "SspiHandshake"
                    && node.tag_name().namespace() == Some(SSAS_NS)
            })
            .ok_or_else(|| {
                XmlaError::ProtocolError("SspiHandshake missing from response".into())
            })?;
        let handshake = handshake_node.text().unwrap_or("").trim();
        if !handshake.is_empty() {
            return Err(XmlaError::ProtocolError(
                "SspiHandshake should be empty".into(),
            ));
        }
        Ok(ntlm)
    }

    fn send_empty_soap_message(stream: &mut TcpStream) -> Result<()> {
        // Sends an empty soap message, if in the configured port there's a SAAS Server
        // running, it should reply with an error, complaining the soap message
        // was empty. Any other error means something is wrong.
        let soap = br#"<soap:Envelope xmlns:soap="http://schemas.xmlsoap.org/soap/envelope/">
          <soap:Body>
          </soap:Body>
        </soap:Envelope>"#;
        let request = DimeMessage {
            options: Some(DimeOptions::default()),
            content_type: Some(String::from("text/xml")),
            data: soap.to_vec(),
        };
        request.write_to(stream)?;

        let response = DimeMessage::read_from(stream)?;

        let xml = std::str::from_utf8(&response.data)?;
        let document = roxmltree::Document::parse(xml)?;
        let error_message = document
            .descendants()
            .find(|node| node.is_element() && node.tag_name().name() == "faultcode")
            .and_then(|node| node.text())
            .map(str::trim);
        if let Some(error_message) = error_message {
            if error_message == "XMLAnalysisError.0xc10f000a" {
                Ok(())
            } else {
                Err(XmlaError::ProtocolError(error_message.into()))
            }
        } else {
            Err(XmlaError::ProtocolError(
                "Unexpected response message".into(),
            ))
        }
    }

    fn get_body_child<'a, 'input: 'a>(
        document: &'a roxmltree::Document<'input>,
    ) -> Result<Node<'a, 'input>> {
        let body = document
            .descendants()
            .find(|node| {
                node.is_element()
                    && node.tag_name().name() == "Body"
                    && node.tag_name().namespace()
                        == Some("http://schemas.xmlsoap.org/soap/envelope/")
            })
            .ok_or_else(|| XmlaError::ProtocolError("No body found".to_string()))?;
        let child = body
            .children()
            .find(|node| node.is_element())
            .ok_or_else(|| XmlaError::ProtocolError("Empty body".to_string()))?;
        Ok(child)
    }
}
