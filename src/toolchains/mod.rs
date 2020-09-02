mod clojerl;
mod elixir;
mod erlang;
mod gleam;

#[derive(Debug, Clone, Default)]
pub struct Toolchain {
    clojerl: clojerl::Toolchain,
    elixir: elixir::Toolchain,
    erlang: erlang::Toolchain,
    gleam: gleam::Toolchain,
}

impl Toolchain {
    pub fn clojerl(&self) -> clojerl::Toolchain {
        self.clojerl.clone()
    }
    pub fn elixir(&self) -> elixir::Toolchain {
        self.elixir.clone()
    }
    pub fn erlang(&self) -> erlang::Toolchain {
        self.erlang.clone()
    }
    pub fn gleam(&self) -> gleam::Toolchain {
        self.gleam.clone()
    }
}
