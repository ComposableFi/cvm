# generated by datamodel-codegen:
#   filename:  response_to_get_ibc_ics20_route.json

from __future__ import annotations

from enum import Enum

from pydantic import BaseModel, Extra, Field, conint


class Addr(BaseModel):
    __root__: str = Field(
        ...,
        description="A human readable address.\n\nIn Cosmos, this is typically bech32 encoded. But for multi-chain smart contracts no assumptions should be made other than being UTF-8 encoded and of reasonable length.\n\nThis type represents a validated address. It can be created in the following ways 1. Use `Addr::unchecked(input)` 2. Use `let checked: Addr = deps.api.addr_validate(input)?` 3. Use `let checked: Addr = deps.api.addr_humanize(canonical_addr)?` 4. Deserialize from JSON. This must only be done from JSON that was validated before such as a contract's state. `Addr` must not be used in messages sent by the user because this would result in unvalidated instances.\n\nThis type is immutable. If you really need to mutate it (Really? Are you sure?), create a mutable copy using `let mut mutable = Addr::to_string()` and operate on that `String` instance.",
    )


class AssetId(BaseModel):
    __root__: str = Field(
        ...,
        description='Newtype for XCVM assets ID. Must be unique for each asset and must never change. This ID is an opaque, arbitrary type from the XCVM protocol and no assumption must be made on how it is computed.',
    )


class ChannelId(BaseModel):
    __root__: str


class IbcIcs20Sender(Enum):
    CosmosStargateIbcApplicationsTransferV1MsgTransfer = (
        'CosmosStargateIbcApplicationsTransferV1MsgTransfer'
    )
    CosmWasmStd1_3 = 'CosmWasmStd1_3'


class NetworkId(BaseModel):
    __root__: conint(ge=0) = Field(
        ...,
        description='Newtype for XCVM networks ID. Must be unique for each network and must never change. This ID is an opaque, arbitrary type from the XCVM protocol and no assumption must be made on how it is computed.',
    )


class CosmWasm(BaseModel):
    admin: Addr = Field(..., description='admin of everything')
    contract: Addr
    interpreter_code_id: conint(ge=0) = Field(
        ..., description='CVM interpreter contract code'
    )


class OutpostId3(BaseModel):
    """
    when message is sent to other side, we should identify receiver of some kind
    """

    class Config:
        extra = Extra.forbid

    cosm_wasm: CosmWasm


class OutpostId(BaseModel):
    __root__: OutpostId3 = Field(
        ...,
        description='when message is sent to other side, we should identify receiver of some kind',
    )


class RelativeTimeout3(BaseModel):
    """
    Timeout is relative to the current block timestamp of counter party
    """

    class Config:
        extra = Extra.forbid

    seconds: conint(ge=0)


class RelativeTimeout(BaseModel):
    __root__: RelativeTimeout3 = Field(
        ...,
        description='relative timeout to CW/IBC-rs time. very small, assumed messages are arriving fast enough, like less than hours',
    )


class IbcIcs20ProgramRoute(BaseModel):
    """
    route is used to describe how to send a full program packet to another network
    """

    channel_to_send_over: ChannelId
    counterparty_timeout: RelativeTimeout
    from_network: NetworkId
    from_outpost: Addr
    ibc_ics_20_sender: IbcIcs20Sender
    local_native_denom: str
    on_remote_asset: AssetId
    to_outpost: OutpostId = Field(
        ..., description='the contract address of the gateway to send to assets'
    )


class GetIbcIcs20RouteResponse(BaseModel):
    route: IbcIcs20ProgramRoute