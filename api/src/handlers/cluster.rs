//! Cluster API handlers

use axum::{extract::Path, extract::Query, http::StatusCode, Json};

use crate::dto::*;
use crate::response::{api_error, ApiResponse, ApiResult};

#[cfg(windows)]
use clus::{Cluster, Csv, GroupState, ResourceState};

// =============================================================================
// Windows Implementation
// =============================================================================

#[cfg(windows)]
pub async fn cluster_info(Query(params): Query<ClusterNameQuery>) -> ApiResult<String> {
    let cluster = Cluster::open(params.name.as_deref())
        .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    let name = cluster
        .name()
        .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    Ok(Json(ApiResponse::success(name)))
}

#[cfg(windows)]
pub async fn cluster_connect(Path(name): Path<String>) -> ApiResult<String> {
    let cluster = Cluster::open(Some(&name))
        .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    let cluster_name = cluster
        .name()
        .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    Ok(Json(ApiResponse::success(cluster_name)))
}

#[cfg(windows)]
pub async fn cluster_list_nodes(Query(params): Query<ClusterNameQuery>) -> ApiResult<Vec<NodeDto>> {
    let cluster = Cluster::open(params.name.as_deref())
        .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    let nodes = cluster
        .nodes()
        .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

    let dtos: Vec<NodeDto> = nodes
        .iter()
        .map(|n| {
            let state = format!("{:?}", n.state());
            NodeDto {
                name: n.name().to_string(),
                state,
            }
        })
        .collect();
    Ok(Json(ApiResponse::success(dtos)))
}

#[cfg(windows)]
pub async fn cluster_get_node(
    Path(name): Path<String>,
    Query(params): Query<ClusterNameQuery>,
) -> ApiResult<NodeDto> {
    let cluster = Cluster::open(params.name.as_deref())
        .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    let node = cluster
        .open_node(&name)
        .map_err(|e| api_error(StatusCode::NOT_FOUND, &e.to_string()))?;
    let state = format!("{:?}", node.state());
    Ok(Json(ApiResponse::success(NodeDto {
        name: node.name().to_string(),
        state,
    })))
}

#[cfg(windows)]
pub async fn cluster_pause_node(
    Path(name): Path<String>,
    Query(params): Query<ClusterNameQuery>,
) -> ApiResult<&'static str> {
    let cluster = Cluster::open(params.name.as_deref())
        .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    let node = cluster
        .open_node(&name)
        .map_err(|e| api_error(StatusCode::NOT_FOUND, &e.to_string()))?;
    node.pause()
        .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    Ok(Json(ApiResponse::success("ok")))
}

#[cfg(windows)]
pub async fn cluster_resume_node(
    Path(name): Path<String>,
    Query(params): Query<ClusterNameQuery>,
) -> ApiResult<&'static str> {
    let cluster = Cluster::open(params.name.as_deref())
        .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    let node = cluster
        .open_node(&name)
        .map_err(|e| api_error(StatusCode::NOT_FOUND, &e.to_string()))?;
    node.resume()
        .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    Ok(Json(ApiResponse::success("ok")))
}

#[cfg(windows)]
pub async fn cluster_list_groups(
    Query(params): Query<ClusterNameQuery>,
) -> ApiResult<Vec<GroupDto>> {
    let cluster = Cluster::open(params.name.as_deref())
        .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    let groups = cluster
        .groups()
        .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

    let dtos: Vec<GroupDto> = groups
        .iter()
        .map(|g| {
            let (state, owner) = g.state().ok().unwrap_or((GroupState::Unknown(0), None));
            GroupDto {
                name: g.name().to_string(),
                state: format!("{:?}", state),
                owner_node: owner,
            }
        })
        .collect();
    Ok(Json(ApiResponse::success(dtos)))
}

#[cfg(windows)]
pub async fn cluster_get_group(
    Path(name): Path<String>,
    Query(params): Query<ClusterNameQuery>,
) -> ApiResult<GroupDto> {
    let cluster = Cluster::open(params.name.as_deref())
        .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    let group = cluster
        .open_group(&name)
        .map_err(|e| api_error(StatusCode::NOT_FOUND, &e.to_string()))?;
    let (state, owner) = group.state().ok().unwrap_or((GroupState::Unknown(0), None));
    Ok(Json(ApiResponse::success(GroupDto {
        name: group.name().to_string(),
        state: format!("{:?}", state),
        owner_node: owner,
    })))
}

#[cfg(windows)]
pub async fn cluster_group_online(
    Path(name): Path<String>,
    Query(params): Query<ClusterNameQuery>,
) -> ApiResult<&'static str> {
    let cluster = Cluster::open(params.name.as_deref())
        .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    let group = cluster
        .open_group(&name)
        .map_err(|e| api_error(StatusCode::NOT_FOUND, &e.to_string()))?;
    group
        .online()
        .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    Ok(Json(ApiResponse::success("ok")))
}

#[cfg(windows)]
pub async fn cluster_group_offline(
    Path(name): Path<String>,
    Query(params): Query<ClusterNameQuery>,
) -> ApiResult<&'static str> {
    let cluster = Cluster::open(params.name.as_deref())
        .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    let group = cluster
        .open_group(&name)
        .map_err(|e| api_error(StatusCode::NOT_FOUND, &e.to_string()))?;
    group
        .offline()
        .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    Ok(Json(ApiResponse::success("ok")))
}

#[cfg(windows)]
pub async fn cluster_move_group(
    Path((name, target_node)): Path<(String, String)>,
    Query(params): Query<ClusterNameQuery>,
) -> ApiResult<&'static str> {
    let cluster = Cluster::open(params.name.as_deref())
        .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    let group = cluster
        .open_group(&name)
        .map_err(|e| api_error(StatusCode::NOT_FOUND, &e.to_string()))?;
    let node = cluster
        .open_node(&target_node)
        .map_err(|e| api_error(StatusCode::NOT_FOUND, &e.to_string()))?;
    group
        .move_to(&node)
        .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    Ok(Json(ApiResponse::success("ok")))
}

#[cfg(windows)]
pub async fn cluster_list_resources(
    Query(params): Query<ClusterNameQuery>,
) -> ApiResult<Vec<ResourceDto>> {
    let cluster = Cluster::open(params.name.as_deref())
        .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    let resources = cluster
        .resources()
        .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

    let dtos: Vec<ResourceDto> = resources
        .iter()
        .map(|r| {
            let (state, owner) = r.state().ok().unwrap_or((ResourceState::Unknown(0), None));
            ResourceDto {
                name: r.name().to_string(),
                state: format!("{:?}", state),
                owner_node: owner,
            }
        })
        .collect();
    Ok(Json(ApiResponse::success(dtos)))
}

#[cfg(windows)]
pub async fn cluster_get_resource(
    Path(name): Path<String>,
    Query(params): Query<ClusterNameQuery>,
) -> ApiResult<ResourceDto> {
    let cluster = Cluster::open(params.name.as_deref())
        .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    let resource = cluster
        .open_resource(&name)
        .map_err(|e| api_error(StatusCode::NOT_FOUND, &e.to_string()))?;
    let (state, owner) = resource
        .state()
        .ok()
        .unwrap_or((ResourceState::Unknown(0), None));
    Ok(Json(ApiResponse::success(ResourceDto {
        name: resource.name().to_string(),
        state: format!("{:?}", state),
        owner_node: owner,
    })))
}

#[cfg(windows)]
pub async fn cluster_resource_online(
    Path(name): Path<String>,
    Query(params): Query<ClusterNameQuery>,
) -> ApiResult<&'static str> {
    let cluster = Cluster::open(params.name.as_deref())
        .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    let resource = cluster
        .open_resource(&name)
        .map_err(|e| api_error(StatusCode::NOT_FOUND, &e.to_string()))?;
    resource
        .online()
        .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    Ok(Json(ApiResponse::success("ok")))
}

#[cfg(windows)]
pub async fn cluster_resource_offline(
    Path(name): Path<String>,
    Query(params): Query<ClusterNameQuery>,
) -> ApiResult<&'static str> {
    let cluster = Cluster::open(params.name.as_deref())
        .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    let resource = cluster
        .open_resource(&name)
        .map_err(|e| api_error(StatusCode::NOT_FOUND, &e.to_string()))?;
    resource
        .offline()
        .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    Ok(Json(ApiResponse::success("ok")))
}

#[cfg(windows)]
pub async fn cluster_list_csv(Query(params): Query<ClusterNameQuery>) -> ApiResult<Vec<CsvDto>> {
    let cluster = Cluster::open(params.name.as_deref())
        .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    let resources = cluster
        .resources()
        .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

    let mut dtos = Vec::new();
    for r in &resources {
        let is_csv = Csv::is_csv_resource(r).unwrap_or(false);
        if is_csv {
            let (state, owner) = r.state().ok().unwrap_or((ResourceState::Unknown(0), None));
            dtos.push(CsvDto {
                name: r.name().to_string(),
                state: format!("{:?}", state),
                owner_node: owner,
                is_csv: true,
            });
        }
    }
    Ok(Json(ApiResponse::success(dtos)))
}

#[cfg(windows)]
pub async fn cluster_csv_check_path(Query(params): Query<CsvPathQuery>) -> ApiResult<bool> {
    let is_csv = Csv::is_path_on_csv(&params.path);
    Ok(Json(ApiResponse::success(is_csv)))
}

#[cfg(windows)]
pub async fn cluster_csv_maintenance(
    Path(name): Path<String>,
    Query(params): Query<ClusterNameQuery>,
    Json(req): Json<MaintenanceModeRequest>,
) -> ApiResult<&'static str> {
    let cluster = Cluster::open(params.name.as_deref())
        .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    let resource = cluster
        .open_resource(&name)
        .map_err(|e| api_error(StatusCode::NOT_FOUND, &e.to_string()))?;
    Csv::set_maintenance_mode(&resource, req.enable)
        .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    Ok(Json(ApiResponse::success("ok")))
}

// =============================================================================
// Non-Windows Stubs
// =============================================================================

#[cfg(not(windows))]
fn not_supported() -> (StatusCode, Json<ApiResponse<()>>) {
    api_error(
        StatusCode::NOT_IMPLEMENTED,
        "Cluster API only available on Windows",
    )
}

#[cfg(not(windows))]
pub async fn cluster_info(_: Query<ClusterNameQuery>) -> ApiResult<String> {
    Err(not_supported())
}

#[cfg(not(windows))]
pub async fn cluster_connect(_: Path<String>) -> ApiResult<String> {
    Err(not_supported())
}

#[cfg(not(windows))]
pub async fn cluster_list_nodes(_: Query<ClusterNameQuery>) -> ApiResult<Vec<NodeDto>> {
    Err(not_supported())
}

#[cfg(not(windows))]
pub async fn cluster_get_node(_: Path<String>, _: Query<ClusterNameQuery>) -> ApiResult<NodeDto> {
    Err(not_supported())
}

#[cfg(not(windows))]
pub async fn cluster_pause_node(
    _: Path<String>,
    _: Query<ClusterNameQuery>,
) -> ApiResult<&'static str> {
    Err(not_supported())
}

#[cfg(not(windows))]
pub async fn cluster_resume_node(
    _: Path<String>,
    _: Query<ClusterNameQuery>,
) -> ApiResult<&'static str> {
    Err(not_supported())
}

#[cfg(not(windows))]
pub async fn cluster_list_groups(_: Query<ClusterNameQuery>) -> ApiResult<Vec<GroupDto>> {
    Err(not_supported())
}

#[cfg(not(windows))]
pub async fn cluster_get_group(_: Path<String>, _: Query<ClusterNameQuery>) -> ApiResult<GroupDto> {
    Err(not_supported())
}

#[cfg(not(windows))]
pub async fn cluster_group_online(
    _: Path<String>,
    _: Query<ClusterNameQuery>,
) -> ApiResult<&'static str> {
    Err(not_supported())
}

#[cfg(not(windows))]
pub async fn cluster_group_offline(
    _: Path<String>,
    _: Query<ClusterNameQuery>,
) -> ApiResult<&'static str> {
    Err(not_supported())
}

#[cfg(not(windows))]
pub async fn cluster_move_group(
    _: Path<(String, String)>,
    _: Query<ClusterNameQuery>,
) -> ApiResult<&'static str> {
    Err(not_supported())
}

#[cfg(not(windows))]
pub async fn cluster_list_resources(_: Query<ClusterNameQuery>) -> ApiResult<Vec<ResourceDto>> {
    Err(not_supported())
}

#[cfg(not(windows))]
pub async fn cluster_get_resource(
    _: Path<String>,
    _: Query<ClusterNameQuery>,
) -> ApiResult<ResourceDto> {
    Err(not_supported())
}

#[cfg(not(windows))]
pub async fn cluster_resource_online(
    _: Path<String>,
    _: Query<ClusterNameQuery>,
) -> ApiResult<&'static str> {
    Err(not_supported())
}

#[cfg(not(windows))]
pub async fn cluster_resource_offline(
    _: Path<String>,
    _: Query<ClusterNameQuery>,
) -> ApiResult<&'static str> {
    Err(not_supported())
}

#[cfg(not(windows))]
pub async fn cluster_list_csv(_: Query<ClusterNameQuery>) -> ApiResult<Vec<CsvDto>> {
    Err(not_supported())
}

#[cfg(not(windows))]
pub async fn cluster_csv_check_path(_: Query<CsvPathQuery>) -> ApiResult<bool> {
    Err(not_supported())
}

#[cfg(not(windows))]
pub async fn cluster_csv_maintenance(
    _: Path<String>,
    _: Query<ClusterNameQuery>,
    _: Json<MaintenanceModeRequest>,
) -> ApiResult<&'static str> {
    Err(not_supported())
}
