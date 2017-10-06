import * as ffi from 'ffi'
import * as ref from 'ref'
import * as Struct from 'ref-struct'
import { CXSRuntime } from '../index'
import { CXSRuntimeConfig, CxsStatus, FFI_CXS_STATUS_PTR } from '../rustlib'

import {
    IConnections
} from './api'

export class Connection implements IConnections {
  public connectionHandle: ref.types.uint32
  public state: ref.types.uint32
  public statusList: any
  readonly RUST_API: ffi

  constructor ( path: string ) {
    this.RUST_API = new CXSRuntime(new CXSRuntimeConfig(path)).ffi
  }

  create ( recipientInfo: string ): number {
    const connectionHandlePtr = ref.alloc(ref.types.uint32)
    const result = this.RUST_API.cxs_connection_create(recipientInfo, connectionHandlePtr)
    this.connectionHandle = ref.deref(connectionHandlePtr, ref.types.uint32)
    return result
  }

  connect (): number {
    return this.RUST_API.cxs_connection_connect(this.connectionHandle)
  }

  get_data (): string {
    return this.RUST_API.cxs_connection_get_data(this.connectionHandle)
  }

  get_state (): number {
    const statusPtr = ref.alloc(ref.types.uint32)
    const result = this.RUST_API.cxs_connection_get_state(this.connectionHandle, statusPtr)
    this.state = ref.deref(statusPtr, ref.types.uint32)
    return result
  }

  release (): number {
    return this.RUST_API.cxs_connection_release(this.connectionHandle)
  }

  list_state (): number {
    const CxsStatusPtr = ref.alloc(FFI_CXS_STATUS_PTR)
    const result = this.RUST_API.cxs_connection_list_state(CxsStatusPtr)
    this.statusList = ref.deref(CxsStatusPtr, FFI_CXS_STATUS_PTR)
    return result
  }
}
