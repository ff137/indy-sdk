extern crate serde_json;

use indy_context::IndyContext;
use command_executor::{Command, CommandMetadata, Group as GroupTrait, GroupMetadata};
use commands::{get_str_param, get_opt_str_param};

use libindy::ErrorCode;
use libindy::pool::Pool;

use std::collections::HashMap;
use std::rc::Rc;

use prettytable::Table;

pub struct Group {
    metadata: GroupMetadata
}

impl Group {
    pub fn new() -> Group {
        Group {
            metadata: GroupMetadata::new("pool", "Pool management commands")
        }
    }
}

impl GroupTrait for Group {
    fn metadata(&self) -> &GroupMetadata {
        &self.metadata
    }
}

#[derive(Debug)]
pub struct CreateCommand {
    ctx: Rc<IndyContext>,
    metadata: CommandMetadata,
}

#[derive(Debug)]
pub struct ConnectCommand {
    ctx: Rc<IndyContext>,
    metadata: CommandMetadata,
}

#[derive(Debug)]
pub struct ListCommand {
    ctx: Rc<IndyContext>,
    metadata: CommandMetadata,
}

#[derive(Debug)]
pub struct DisconnectCommand {
    ctx: Rc<IndyContext>,
    metadata: CommandMetadata,
}

#[derive(Debug)]
pub struct DeleteCommand {
    ctx: Rc<IndyContext>,
    metadata: CommandMetadata,
}


impl CreateCommand {
    pub fn new(ctx: Rc<IndyContext>) -> CreateCommand {
        CreateCommand {
            ctx,
            metadata: CommandMetadata::build("create", "Create new pool ledger config with specified name")
                .add_main_param("name", "The name of new pool ledger config")
                .add_param("gen_txn_file", true, "Path to file with genesis transactions")
                .finalize()
        }
    }
}

impl Command for CreateCommand {
    fn execute(&self, params: &HashMap<&'static str, &str>) -> Result<(), ()> {
        trace!("CreateCommand::execute >> self {:?} params {:?}", self, params);

        let name = get_str_param("name", params).map_err(error_err!())?;
        let gen_txn_file = get_opt_str_param("gen_txn_file", params).map_err(error_err!())?
            .unwrap_or("pool_transactions_genesis");

        let config: String = json!({ "genesis_txn": gen_txn_file }).to_string();

        trace!(r#"Pool::create_pool_ledger_config try: name {}, config {:?}"#, name, config);

        let res = Pool::create_pool_ledger_config(name,
                                                  config.as_str());

        trace!(r#"Pool::create_pool_ledger_config return: {:?}"#, res);

        let res = match res {
            Ok(()) => Ok(println_succ!("Pool config \"{}\" has been created", name)),
            Err(ErrorCode::PoolLedgerConfigAlreadyExistsError) => Err(println_err!("Pool config \"{}\" already exists", name)),
            Err(err) => Err(println_err!("Indy SDK error occurred {:?}", err)),
        };

        trace!("CreateCommand::execute << {:?}", res);
        res
    }

    fn metadata(&self) -> &CommandMetadata {
        &self.metadata
    }
}

impl ConnectCommand {
    pub fn new(ctx: Rc<IndyContext>) -> ConnectCommand {
        ConnectCommand {
            ctx,
            metadata: CommandMetadata::build("connect", "Connect to pool with specified name. Also disconnect from previously connected.")
                .add_main_param("name", "The name of pool")
                .finalize()
        }
    }
}

impl Command for ConnectCommand {
    fn metadata(&self) -> &CommandMetadata {
        &self.metadata
    }

    fn execute(&self, params: &HashMap<&'static str, &str>) -> Result<(), ()> {
        trace!("OpenCommand::execute >> self {:?} params {:?}", self, params);

        let name = get_str_param("name", params).map_err(error_err!())?;

        //TODO close previously opened pool
        let res = match Pool::open_pool_ledger(name, None) {
            Ok(handle) => {
                self.ctx.set_connected_pool(name, handle);
                Ok(println_succ!("Pool \"{}\" has been connected", name))
            }
            Err(err) => Err(println_err!("Indy SDK error occurred {:?}", err)),
        };

        trace!("CreateCommand::execute << {:?}", res);
        Ok(())
    }
}

impl ListCommand {
    pub fn new(ctx: Rc<IndyContext>) -> ListCommand {
        ListCommand {
            ctx,
            metadata: CommandMetadata::build("list", "List existing pool configs.")
                .finalize()
        }
    }
}

impl Command for ListCommand {
    fn metadata(&self) -> &CommandMetadata {
        &self.metadata
    }

    fn execute(&self, params: &HashMap<&'static str, &str>) -> Result<(), ()> {
        trace!("ListCommand::execute >> self {:?} params {:?}", self, params);

        let res = match Pool::list() {
            Ok(pools) => {
                let pools: Vec<String> = serde_json::from_str(&pools)
                    .map_err(|_| println_err!("Wrong data has been received"))?;

                let mut table = Table::new();
                table.add_row(row![Fgb->"name"]);
                for pool in pools {
                    table.add_row(row![pool]);
                }
                table.printstd();


                Ok(())
            }
            Err(ErrorCode::CommonIOError) => Err(println_succ!("There are no pool")),
            Err(err) => Err(println_err!("Indy SDK error occurred {:?}", err)),
        };

        trace!("ListCommand::execute << {:?}", res);
        res
    }
}

impl DisconnectCommand {
    pub fn new(ctx: Rc<IndyContext>) -> DisconnectCommand {
        DisconnectCommand {
            ctx,
            metadata: CommandMetadata::build("disconnect", "Disconnect from current pool.")
                .finalize()
        }
    }
}

impl Command for DisconnectCommand {
    fn metadata(&self) -> &CommandMetadata {
        &self.metadata
    }

    fn execute(&self, params: &HashMap<&'static str, &str>) -> Result<(), ()> {
        trace!("DisconnectCommand::execute >> self {:?} params {:?}", self, params);

        let (name, handle) = if let Some(pool) = self.ctx.get_connected_pool() {
            pool
        } else {
            return Err(println_err!("There is no connected pool now"));
        };

        let res = match Pool::close(handle) {
            Ok(()) => {
                self.ctx.unset_connected_pool();
                Ok(println_succ!("Pool \"{}\" has been disconnected", name))
            }
            Err(err) => Err(println_err!("Indy SDK error occurred {:?}", err)),
        };

        trace!("DisconnectCommand::execute << {:?}", res);
        res
    }
}

impl DeleteCommand {
    pub fn new(ctx: Rc<IndyContext>) -> DeleteCommand {
        DeleteCommand {
            ctx,
            metadata: CommandMetadata::build("delete", "Delete pool config with specified name")
                .add_main_param("name", "The name of deleted pool config")
                .finalize()
        }
    }
}

impl Command for DeleteCommand {
    fn execute(&self, params: &HashMap<&'static str, &str>) -> Result<(), ()> {
        trace!("DeleteCommand::execute >> self {:?} params {:?}", self, params);

        let name = get_str_param("name", params).map_err(error_err!())?;

        trace!(r#"Pool::delete try: name {}"#, name);

        let res = Pool::delete(name);

        trace!(r#"Pool::delete return: {:?}"#, res);

        let res = match res {
            Ok(()) => Ok(println_succ!("Pool \"{}\" has been deleted", name)),
            Err(err) => Err(println_err!("Indy SDK error occurred {:?}", err)),
        };

        trace!("DeleteCommand::execute << {:?}", res);
        res
    }

    fn metadata(&self) -> &CommandMetadata {
        &self.metadata
    }
}