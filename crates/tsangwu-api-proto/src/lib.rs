pub mod common {
    tonic::include_proto!("tsangwu.common");
}

pub mod user {
    tonic::include_proto!("tsangwu.user");
}

pub mod auth {
    tonic::include_proto!("tsangwu.auth");
}

pub mod project {
    tonic::include_proto!("tsangwu.project");
}

pub mod asset {
    tonic::include_proto!("tsangwu.asset");
}

pub mod generation {
    tonic::include_proto!("tsangwu.generation");
}

pub mod agent {
    tonic::include_proto!("tsangwu.agent");
}

pub mod culture {
    tonic::include_proto!("tsangwu.culture");
}
