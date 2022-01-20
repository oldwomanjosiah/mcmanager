use data::auth::*;
use data::prelude::*;
use tonic::{Request, Response, Status};

use crate::manager::AuthManager;

pub struct AuthService {
    pub auth_manager: AuthManager,
}

impl AuthService {
    async fn refresh(
        &self,
        _request: Request<RefreshRequest>,
    ) -> Result<Response<AuthResponse>, Status> {
        unimplemented!()
    }

    async fn authenticate(
        &self,
        request: Request<AuthRequest>,
    ) -> Result<Response<AuthResponse>, Status> {
        let request = request.into_inner();
        let username = request.username;
        let password = request.password;

        let auth = self
            .auth_manager
            .authorize_user(&username, &password)
            .await
            .into();

        Ok(AuthResponse {
            authorization: Some(auth),
        }
        .into_msg())
    }
}

#[tonic::async_trait]
impl Auth for AuthService {
    async fn refresh(
        &self,
        request: Request<RefreshRequest>,
    ) -> Result<Response<AuthResponse>, Status> {
        self.refresh(request).await
    }

    async fn authenticate(
        &self,
        request: Request<AuthRequest>,
    ) -> Result<Response<AuthResponse>, Status> {
        self.authenticate(request).await
    }
}