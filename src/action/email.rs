use super::Action;
use crate::config;
use crate::{Error, PlaceholderMap, Result};
use async_trait::async_trait;

pub struct Email {
    from: lettre::message::Mailbox,
    to: lettre::message::Mailbox,
    reply_to: Option<lettre::message::Mailbox>,
    subject: String,
    body: String,
    smtp_server: String,
    smtp_port: Option<u16>,
    smtp_security: config::SmtpSecurity,
    username: String,
    password: String,
}

impl TryFrom<&config::Action> for Email {
    type Error = Error;

    fn try_from(action: &config::Action) -> std::result::Result<Self, Self::Error> {
        if let config::ActionType::Email(email) = &action.type_ {
            if email.subject.is_empty() {
                Err(Error(String::from("'subject' cannot be empty.")))
            } else if email.body.is_empty() {
                Err(Error(String::from("'body' cannot be empty.")))
            } else if email.smtp_server.is_empty() {
                Err(Error(String::from("'smtp_server' cannot be empty.")))
            } else if email.username.is_empty() {
                Err(Error(String::from("'username' cannot be empty.")))
            } else if email.password.is_empty() {
                Err(Error(String::from("'password' cannot be empty.")))
            } else {
                Ok(Self {
                    from: email
                        .from
                        .parse()
                        .map_err(|x| Error(format!("Invalid sender email address: {}", x)))?,
                    to: email
                        .to
                        .parse()
                        .map_err(|x| Error(format!("Invalid recipient email address: {}", x)))?,
                    reply_to: email.reply_to.as_ref().map_or(Ok(None), |x| {
                        Ok(Some(x.parse().map_err(|x| {
                            Error(format!("Invalid reply-to email address: {}", x))
                        })?))
                    })?,
                    subject: email.subject.clone(),
                    body: email.body.clone(),
                    smtp_server: email.smtp_server.clone(),
                    smtp_port: email.smtp_port,
                    smtp_security: email.smtp_security,
                    username: email.username.clone(),
                    password: email.password.clone(),
                })
            }
        } else {
            panic!();
        }
    }
}

#[async_trait]
impl Action for Email {
    async fn trigger(&self, placeholders: PlaceholderMap) -> Result<()> {
        use lettre::transport::smtp::authentication::Credentials;
        use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};

        let subject = crate::fill_placeholders(self.subject.as_str(), &placeholders);
        let body = crate::fill_placeholders(self.body.as_str(), &placeholders);
        let mut message_builder = Message::builder()
            .from(self.from.clone())
            .to(self.to.clone())
            .subject(&subject)
            .user_agent(crate::user_agent());
        if let Some(reply_to) = &self.reply_to {
            message_builder = message_builder.reply_to(reply_to.clone());
        }
        let email = message_builder
            .body(body)
            .map_err(|x| Error(x.to_string()))?;
        let credentials = Credentials::new(self.username.clone(), self.password.clone());
        let mut mailer_builder = match self.smtp_security {
            config::SmtpSecurity::TLS => {
                AsyncSmtpTransport::<Tokio1Executor>::relay(&self.smtp_server)
                    .map_err(|x| Error(x.to_string()))
            }
            config::SmtpSecurity::STARTTLS => {
                AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&self.smtp_server)
                    .map_err(|x| Error(x.to_string()))
            }
            config::SmtpSecurity::Plain => Ok(
                AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(&self.smtp_server),
            ),
        }?;

        mailer_builder = mailer_builder.credentials(credentials);
        if let Some(port) = self.smtp_port {
            mailer_builder = mailer_builder.port(port);
        }
        let mailer = mailer_builder.build();
        mailer
            .send(email)
            .await
            .map_err(|x| Error(format!("Failed to send email: {}", x)))?;
        Ok(())
    }
}
