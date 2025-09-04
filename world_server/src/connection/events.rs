use std::{fmt, net::SocketAddr};

use wow_world_messages::wrath::{opcodes::ClientOpcodeMessage, *};

/// Events produced by the network/IO layer and consumed by the client manager.
#[allow(clippy::large_enum_variant)]
pub enum ClientEvent {
    Connected {
        addr: SocketAddr,
        account_id: u32,
        // This sender is used to send messages back to the client from the manager
        connection_sender: flume::Sender<ServerEvent>,
    },
    Disconnected {
        addr: SocketAddr,
    },
    Message {
        addr: SocketAddr,
        packet: ClientOpcodeMessage,
    },
}

/// Events sent by the client manager back to the connection writer for delivery to the client.
#[derive(Clone)]
pub enum ServerEvent {
    AccountDataTimes(SMSG_ACCOUNT_DATA_TIMES),
    ActionButtons(SMSG_ACTION_BUTTONS),
    BindPointUpdate(SMSG_BINDPOINTUPDATE),
    CalendarSendNumPending(SMSG_CALENDAR_SEND_NUM_PENDING),
    CharCreate(SMSG_CHAR_CREATE),
    CharDelete(SMSG_CHAR_DELETE),
    CharEnum(SMSG_CHAR_ENUM),
    ContactList(SMSG_CONTACT_LIST),
    DestroyObject(SMSG_DESTROY_OBJECT),
    Disconnect,
    FeatureSystemStatus(SMSG_FEATURE_SYSTEM_STATUS),
    ForceMoveRoot(SMSG_FORCE_MOVE_ROOT),
    ForceMoveUnroot(SMSG_FORCE_MOVE_UNROOT),
    GMTicketGetTicket(SMSG_GMTICKET_GETTICKET),
    GMTicketSystemStatus(SMSG_GMTICKET_SYSTEMSTATUS),
    InitializeFactions(SMSG_INITIALIZE_FACTIONS),
    InitialSpells(SMSG_INITIAL_SPELLS),
    InitWorldStates(SMSG_INIT_WORLD_STATES),
    ItemNameQueryResponse(SMSG_ITEM_NAME_QUERY_RESPONSE),
    ItemQuerySingleResponse(SMSG_ITEM_QUERY_SINGLE_RESPONSE),
    LoginSetTimeSpeed(SMSG_LOGIN_SETTIMESPEED),
    LoginVerifyWorld(SMSG_LOGIN_VERIFY_WORLD),
    LogoutCancelAck(SMSG_LOGOUT_CANCEL_ACK),
    LogoutComplete(SMSG_LOGOUT_COMPLETE),
    LogoutResponse(SMSG_LOGOUT_RESPONSE),
    MessageChat(SMSG_MESSAGECHAT),
    MoveTeleportAck(MSG_MOVE_TELEPORT_ACK_Server),
    MoveStartForward(MSG_MOVE_START_FORWARD),
    MoveStartBackward(MSG_MOVE_START_BACKWARD),
    MoveStop(MSG_MOVE_STOP),
    MoveStopTurn(MSG_MOVE_STOP_TURN),
    MoveStartStrafeLeft(MSG_MOVE_START_STRAFE_LEFT),
    MoveStartStrafeRight(MSG_MOVE_START_STRAFE_RIGHT),
    MoveStopStrafe(MSG_MOVE_STOP_STRAFE),
    MoveJump(MSG_MOVE_JUMP),
    MoveStartTurnLeft(MSG_MOVE_START_TURN_LEFT),
    MoveStartTurnRight(MSG_MOVE_START_TURN_RIGHT),
    MoveStartPitchUp(MSG_MOVE_START_PITCH_UP),
    MoveStartPitchDown(MSG_MOVE_START_PITCH_DOWN),
    MoveStopPitch(MSG_MOVE_STOP_PITCH),
    MoveSetRunMode(MSG_MOVE_SET_RUN_MODE),
    MoveSetWalkMode(MSG_MOVE_SET_WALK_MODE),
    MoveFallLand(MSG_MOVE_FALL_LAND),
    MoveStartSwim(MSG_MOVE_START_SWIM),
    MoveStopSwim(MSG_MOVE_STOP_SWIM),
    MoveSetFacing(MSG_MOVE_SET_FACING),
    MoveHeartbeat(MSG_MOVE_HEARTBEAT),
    NameQueryResponse(SMSG_NAME_QUERY_RESPONSE),
    NewWorld(SMSG_NEW_WORLD),
    PlayedTime(SMSG_PLAYED_TIME),
    QueryTimeResponse(SMSG_QUERY_TIME_RESPONSE),
    Pong(SMSG_PONG),
    RaidInstanceInfo(SMSG_RAID_INSTANCE_INFO),
    RealmSplit(SMSG_REALM_SPLIT),
    SetDungeonDifficulty(MSG_SET_DUNGEON_DIFFICULTY_Server),
    StandStateUpdate(SMSG_STANDSTATE_UPDATE),
    TimeSyncReq(SMSG_TIME_SYNC_REQ),
    TransferPending(SMSG_TRANSFER_PENDING),
    TriggerCinematic(SMSG_TRIGGER_CINEMATIC),
    TutorialFlags(SMSG_TUTORIAL_FLAGS),
    UpdateAccountData(SMSG_UPDATE_ACCOUNT_DATA),
    UpdateAccountDataComplete(SMSG_UPDATE_ACCOUNT_DATA_COMPLETE),
    UpdateObject(SMSG_UPDATE_OBJECT),
    UpdateWorldState(SMSG_UPDATE_WORLD_STATE),
    WorldStateUiTimerUpdate(SMSG_WORLD_STATE_UI_TIMER_UPDATE),
}

impl fmt::Display for ServerEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ServerEvent::AccountDataTimes(_) => write!(f, "SMSG_ACCOUNT_DATA_TIMES"),
            ServerEvent::ActionButtons(_) => write!(f, "SMSG_ACTION_BUTTONS"),
            ServerEvent::BindPointUpdate(_) => write!(f, "SMSG_BINDPOINTUPDATE"),
            ServerEvent::CalendarSendNumPending(_) => write!(f, "SMSG_CALENDAR_SEND_NUM_PENDING"),
            ServerEvent::CharCreate(_) => write!(f, "SMSG_CHAR_CREATE"),
            ServerEvent::CharDelete(_) => write!(f, "SMSG_CHAR_DELETE"),
            ServerEvent::CharEnum(_) => write!(f, "SMSG_CHAR_ENUM"),
            ServerEvent::ContactList(_) => write!(f, "SMSG_CONTACT_LIST"),
            ServerEvent::DestroyObject(_) => write!(f, "SMSG_DESTROY_OBJECT"),
            ServerEvent::Disconnect => write!(f, "Disconnect"),
            ServerEvent::FeatureSystemStatus(_) => write!(f, "SMSG_FEATURE_SYSTEM_STATUS"),
            ServerEvent::ForceMoveRoot(_) => write!(f, "SMSG_FORCE_MOVE_ROOT"),
            ServerEvent::ForceMoveUnroot(_) => write!(f, "SMSG_FORCE_MOVE_UNROOT"),
            ServerEvent::GMTicketGetTicket(_) => write!(f, "SMSG_GMTICKET_GETTICKET"),
            ServerEvent::GMTicketSystemStatus(_) => write!(f, "SMSG_GMTICKET_SYSTEMSTATUS"),
            ServerEvent::InitializeFactions(_) => write!(f, "SMSG_INITIALIZE_FACTIONS"),
            ServerEvent::InitialSpells(_) => write!(f, "SMSG_INITIAL_SPELLS"),
            ServerEvent::InitWorldStates(_) => write!(f, "SMSG_INIT_WORLD_STATES"),
            ServerEvent::ItemNameQueryResponse(_) => write!(f, "SMSG_ITEM_NAME_QUERY_RESPONSE"),
            ServerEvent::ItemQuerySingleResponse(_) => write!(f, "SMSG_ITEM_QUERY_SINGLE_RESPONSE"),
            ServerEvent::LoginSetTimeSpeed(_) => write!(f, "SMSG_LOGIN_SETTIMESPEED"),
            ServerEvent::LoginVerifyWorld(_) => write!(f, "SMSG_LOGIN_VERIFY_WORLD"),
            ServerEvent::LogoutCancelAck(_) => write!(f, "SMSG_LOGOUT_CANCEL_ACK"),
            ServerEvent::LogoutComplete(_) => write!(f, "SMSG_LOGOUT_COMPLETE"),
            ServerEvent::LogoutResponse(_) => write!(f, "SMSG_LOGOUT_RESPONSE"),
            ServerEvent::MessageChat(_) => write!(f, "SMSG_MESSAGECHAT"),
            ServerEvent::MoveTeleportAck(_) => write!(f, "MSG_MOVE_TELEPORT_ACK_Server"),
            ServerEvent::MoveStartForward(_) => write!(f, "MSG_MOVE_START_FORWARD"),
            ServerEvent::MoveStartBackward(_) => write!(f, "MSG_MOVE_START_BACKWARD"),
            ServerEvent::MoveStop(_) => write!(f, "MSG_MOVE_STOP"),
            ServerEvent::MoveStopTurn(_) => write!(f, "MSG_MOVE_STOP_TURN"),
            ServerEvent::MoveStartStrafeLeft(_) => write!(f, "MSG_MOVE_START_STRAFE_LEFT"),
            ServerEvent::MoveStartStrafeRight(_) => write!(f, "MSG_MOVE_START_STRAFE_RIGHT"),
            ServerEvent::MoveStopStrafe(_) => write!(f, "MSG_MOVE_STOP_STRAFE"),
            ServerEvent::MoveJump(_) => write!(f, "MSG_MOVE_JUMP"),
            ServerEvent::MoveStartTurnLeft(_) => write!(f, "MSG_MOVE_START_TURN_LEFT"),
            ServerEvent::MoveStartTurnRight(_) => write!(f, "MSG_MOVE_START_TURN_RIGHT"),
            ServerEvent::MoveStartPitchUp(_) => write!(f, "MSG_MOVE_START_PITCH_UP"),
            ServerEvent::MoveStartPitchDown(_) => write!(f, "MSG_MOVE_START_PITCH_DOWN"),
            ServerEvent::MoveStopPitch(_) => write!(f, "MSG_MOVE_STOP_PITCH"),
            ServerEvent::MoveSetRunMode(_) => write!(f, "MSG_MOVE_SET_RUN_MODE"),
            ServerEvent::MoveSetWalkMode(_) => write!(f, "MSG_MOVE_SET_WALK_MODE"),
            ServerEvent::MoveFallLand(_) => write!(f, "MSG_MOVE_FALL_LAND"),
            ServerEvent::MoveStartSwim(_) => write!(f, "MSG_MOVE_START_SWIM"),
            ServerEvent::MoveStopSwim(_) => write!(f, "MSG_MOVE_STOP_SWIM"),
            ServerEvent::MoveSetFacing(_) => write!(f, "MSG_MOVE_SET_FACING"),
            ServerEvent::MoveHeartbeat(_) => write!(f, "MSG_MOVE_HEARTBEAT"),
            ServerEvent::NameQueryResponse(_) => write!(f, "SMSG_NAME_QUERY_RESPONSE"),
            ServerEvent::NewWorld(_) => write!(f, "SMSG_NEW_WORLD"),
            ServerEvent::PlayedTime(_) => write!(f, "SMSG_PLAYED_TIME"),
            ServerEvent::QueryTimeResponse(_) => write!(f, "SMSG_QUERY_TIME_RESPONSE"),
            ServerEvent::Pong(_) => write!(f, "SMSG_PONG"),
            ServerEvent::RaidInstanceInfo(_) => write!(f, "SMSG_RAID_INSTANCE_INFO"),
            ServerEvent::RealmSplit(_) => write!(f, "SMSG_REALM_SPLIT"),
            ServerEvent::SetDungeonDifficulty(_) => write!(f, "MSG_SET_DUNGEON_DIFFICULTY_Server"),
            ServerEvent::StandStateUpdate(_) => write!(f, "SMSG_STANDSTATE_UPDATE"),
            ServerEvent::TimeSyncReq(_) => write!(f, "SMSG_TIME_SYNC_REQ"),
            ServerEvent::TransferPending(_) => write!(f, "SMSG_TRANSFER_PENDING"),
            ServerEvent::TriggerCinematic(_) => write!(f, "SMSG_TRIGGER_CINEMATIC"),
            ServerEvent::TutorialFlags(_) => write!(f, "SMSG_TUTORIAL_FLAGS"),
            ServerEvent::UpdateAccountData(_) => write!(f, "SMSG_UPDATE_ACCOUNT_DATA"),
            ServerEvent::UpdateAccountDataComplete(_) => write!(f, "SMSG_UPDATE_ACCOUNT_DATA_COMPLETE"),
            ServerEvent::UpdateObject(_) => write!(f, "SMSG_UPDATE_OBJECT"),
            ServerEvent::UpdateWorldState(_) => write!(f, "SMSG_UPDATE_WORLD_STATE"),
            ServerEvent::WorldStateUiTimerUpdate(_) => write!(f, "SMSG_WORLD_STATE_UI_TIMER_UPDATE"),
        }
    }
}

pub trait IntoServerEvent {
    fn into_server_event(self) -> ServerEvent;
}

impl IntoServerEvent for MSG_MOVE_START_FORWARD {
    fn into_server_event(self) -> ServerEvent {
        ServerEvent::MoveStartForward(self)
    }
}

impl IntoServerEvent for MSG_MOVE_START_BACKWARD {
    fn into_server_event(self) -> ServerEvent {
        ServerEvent::MoveStartBackward(self)
    }
}

impl IntoServerEvent for MSG_MOVE_STOP {
    fn into_server_event(self) -> ServerEvent {
        ServerEvent::MoveStop(self)
    }
}

impl IntoServerEvent for MSG_MOVE_STOP_TURN {
    fn into_server_event(self) -> ServerEvent {
        ServerEvent::MoveStopTurn(self)
    }
}

impl IntoServerEvent for MSG_MOVE_START_STRAFE_LEFT {
    fn into_server_event(self) -> ServerEvent {
        ServerEvent::MoveStartStrafeLeft(self)
    }
}

impl IntoServerEvent for MSG_MOVE_START_STRAFE_RIGHT {
    fn into_server_event(self) -> ServerEvent {
        ServerEvent::MoveStartStrafeRight(self)
    }
}

impl IntoServerEvent for MSG_MOVE_STOP_STRAFE {
    fn into_server_event(self) -> ServerEvent {
        ServerEvent::MoveStopStrafe(self)
    }
}

impl IntoServerEvent for MSG_MOVE_JUMP {
    fn into_server_event(self) -> ServerEvent {
        ServerEvent::MoveJump(self)
    }
}

impl IntoServerEvent for MSG_MOVE_START_TURN_LEFT {
    fn into_server_event(self) -> ServerEvent {
        ServerEvent::MoveStartTurnLeft(self)
    }
}

impl IntoServerEvent for MSG_MOVE_START_TURN_RIGHT {
    fn into_server_event(self) -> ServerEvent {
        ServerEvent::MoveStartTurnRight(self)
    }
}

impl IntoServerEvent for MSG_MOVE_START_PITCH_UP {
    fn into_server_event(self) -> ServerEvent {
        ServerEvent::MoveStartPitchUp(self)
    }
}

impl IntoServerEvent for MSG_MOVE_START_PITCH_DOWN {
    fn into_server_event(self) -> ServerEvent {
        ServerEvent::MoveStartPitchDown(self)
    }
}

impl IntoServerEvent for MSG_MOVE_STOP_PITCH {
    fn into_server_event(self) -> ServerEvent {
        ServerEvent::MoveStopPitch(self)
    }
}

impl IntoServerEvent for MSG_MOVE_SET_RUN_MODE {
    fn into_server_event(self) -> ServerEvent {
        ServerEvent::MoveSetRunMode(self)
    }
}

impl IntoServerEvent for MSG_MOVE_SET_WALK_MODE {
    fn into_server_event(self) -> ServerEvent {
        ServerEvent::MoveSetWalkMode(self)
    }
}

impl IntoServerEvent for MSG_MOVE_FALL_LAND {
    fn into_server_event(self) -> ServerEvent {
        ServerEvent::MoveFallLand(self)
    }
}

impl IntoServerEvent for MSG_MOVE_START_SWIM {
    fn into_server_event(self) -> ServerEvent {
        ServerEvent::MoveStartSwim(self)
    }
}

impl IntoServerEvent for MSG_MOVE_STOP_SWIM {
    fn into_server_event(self) -> ServerEvent {
        ServerEvent::MoveStopSwim(self)
    }
}

impl IntoServerEvent for MSG_MOVE_SET_FACING {
    fn into_server_event(self) -> ServerEvent {
        ServerEvent::MoveSetFacing(self)
    }
}

impl IntoServerEvent for MSG_MOVE_HEARTBEAT {
    fn into_server_event(self) -> ServerEvent {
        ServerEvent::MoveHeartbeat(self)
    }
}

/// Event multiplexing between client (socket) and server (manager) sides.
#[allow(clippy::large_enum_variant)]
pub enum ConnectionEvent {
    /// Incoming message from the client to the server
    Client(ClientOpcodeMessage),

    /// Outgoing message from the server to the client
    Server(ServerEvent),
}
