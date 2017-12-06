import * as ffi from 'ffi'
import { ConnectionTimeoutError, CXSInternalError } from '../errors'
import { rustAPI } from '../rustlib'
import { createFFICallbackPromise } from '../utils/ffi-helpers'
import { StateType } from './common'
import { CXSBase } from './CXSBase'

/**
 * @description Interface that represents the attributes of a Connection object.
 * This data is expected as the type for deserialize's parameter and serialize's return value
 * @interface
 */
export interface IConnectionData {
  source_id: string
  invite_detail: string,
  handle: number,
  pw_did: string,
  pw_verkey: string,
  did_endpoint: string,
  endpoint: string,
  uuid: string,
  wallet: string,
  state: StateType
}

export interface IRecipientInfo {
  id: string
}

export interface IConnectOptions {
  phone?: string,
  timeout?: number
}

/**
 * @class Class representing a Connection
 */
export class Connection extends CXSBase {
  protected _releaseFn = rustAPI().cxs_connection_release
  protected _updateStFn = rustAPI().cxs_connection_update_state
  protected _serializeFn = rustAPI().cxs_connection_serialize
  protected _deserializeFn = rustAPI().cxs_connection_deserialize

  /**
   * @memberof Connection
   * @description Builds a generic Connection object.
   * @static
   * @async
   * @function create
   * @param {IRecipientInfo} recipientInfo
   * @example <caption>Example of recipientInfo</caption>
   * {id: "123"}
   * @returns {Promise<Connection>} A Connection Object
   */
  static async create ( recipientInfo: IRecipientInfo): Promise<Connection> {
    const connection = new Connection(recipientInfo.id)
    const commandHandle = 0
    try {
      await connection._create((cb) => rustAPI().cxs_connection_create(commandHandle, recipientInfo.id, cb))
      return connection
    } catch (err) {
      throw new CXSInternalError(`cxs_connection_create -> ${err}`)
    }
  }

  /**
   * @memberof Connection
   * @description Builds a Connection object with defined attributes.
   * The attributes are often provided by a previous call to the serialize function
   * @static
   * @async
   * @function deserialize
   * @param {IConnectionData} connectionData - contains the information that will be used to build a connection object
   * @example <caption>Example of Connection Data </caption>
   * {source_id:"234",handle:560373036,pw_did:"did",pw_verkey:"verkey",did_endpoint:"",state:2,uuid:"",endpoint:"",
   * invite_detail:{e:"",rid:"",sakdp:"",sn:"",sD:"",lu:"",sVk:"",tn:""}}
   * @returns {Promise<Connection>} A Connection Object
   */
  static async deserialize (connectionData: IConnectionData) {
    try {
      return await super._deserialize(Connection, connectionData)
    } catch (err) {
      throw new CXSInternalError(`cxs_connection_deserialize -> ${err}`)
    }
  }

  /**
   * @memberof Connection
   * @description Creates a connection between enterprise and end user.
   * @async
   * @function connect
   * @param {IConnectOptions} options - data determining if connection is established by SMS or QR code. Default is SMS
   * @example <caption>Example of IConnectionOptions</caption>
   * { phone: "800", timeout: 30 }
   * @returns {Promise<void>}
   */
  async connect ( options: IConnectOptions = {} ): Promise<void> {
    const timeout = options.timeout || 10000
    await this._waitFor(async () => await this._connect(options) === 0, timeout)
  }

  /**
   * @memberof Connection
   * @description Serializes a connection object.
   * Data returned can be used to recreate a Connection object by passing it to the deserialize function.
   * @async
   * @function serialize
   * @returns {Promise<IConnectionData>} - Jason object with all of the underlying Rust attributes.
   * Same json object structure that is passed to the deserialize function.
   */
  async serialize (): Promise<IConnectionData> {
    try {
      const data: IConnectionData = JSON.parse(await super._serialize())
      return data
    } catch (err) {
      throw new CXSInternalError(`cxs_connection_serialize -> ${err}`)
    }
  }

  /**
   * @memberof Connection
   * @description Communicates with the agent service for polling and setting the state of the Connection.
   * @async
   * @function updateState
   * @returns {Promise<void>}
   */
  async updateState (): Promise<void> {
    try {
      await this._updateState()
    } catch (error) {
      throw new CXSInternalError(`cxs_connection_updateState -> ${error}`)
    }
  }

  private async _connect (options: IConnectOptions): Promise<number> {
    const phone = options.phone
    const connectionType: string = phone ? 'SMS' : 'QR'
    const connectionData: string = JSON.stringify({connection_type: connectionType, phone})
    try {
      return await createFFICallbackPromise<number>(
          (resolve, reject, cb) => {
            const rc = rustAPI().cxs_connection_connect(0, this._handle, connectionData, cb)
            if (rc) {
              resolve(rc)
            }
          },
          (resolve, reject) => ffi.Callback('void', ['uint32', 'uint32'], (xHandle, err) => {
            resolve(err)
          })
        )
    } catch (error) {
      throw new CXSInternalError(`cxs_connection_connect -> ${error}`)
    }
  }

  private _sleep = (sleepTime: number): Promise<void> => new Promise((res) => setTimeout(res, sleepTime))

  private _waitFor = async (predicate: () => any, timeout: number, sleepTime: number = 1000) => {
    if (timeout < 0) {
      throw new ConnectionTimeoutError()
    }
    const res = predicate()
    if (!res) {
      await this._sleep(sleepTime)
      return this._waitFor(predicate, timeout - sleepTime)
    }
    return res
  }
}
