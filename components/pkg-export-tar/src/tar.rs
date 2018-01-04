use std::fs::{self, OpenOptions};
use std::io::Write;

use common::ui::{UI, Status};
use hcore::os::filesystem;
use handlebars::Handlebars;
use build::BuildRoot;

use error::{Error, Result};
use serde_json;

use util;

const INIT_SH: &'static str = include_str!("../defaults/init.sh.hbs");

/// A temporary file system build root for building a tarball, based on Habitat packages.
pub struct TarBuildRoot(BuildRoot);

impl TarBuildRoot {
    pub fn from_build_root(build_root: BuildRoot, ui: &mut UI) -> Result<Self> {
        let root = TarBuildRoot(build_root);
        if cfg!(target_os = "linux") {
            root.add_users_and_groups(ui)?;
            root.create_entrypoint(ui)?;
        }

        //root.create_dockerfile(ui)?;

        Ok(root)
    }

    fn add_users_and_groups(&self, ui: &mut UI) -> Result<()> {
        let ctx = self.0.ctx();
        let (users, groups) = ctx.svc_users_and_groups()?;
        {
            let file = "etc/passwd";
            let mut f = OpenOptions::new().append(true).open(
                ctx.rootfs().join(&file),
            )?;
            for line in users {
                let user = line.split(":").next().expect(
                    "user line contains first entry",
                );
                ui.status(
                    Status::Creating,
                    format!("user '{}' in /{}", user, &file),
                )?;
                f.write_all(line.as_bytes())?;
            }
        }
        {
            let file = "etc/group";
            let mut f = OpenOptions::new().append(true).open(
                ctx.rootfs().join(&file),
            )?;
            for line in groups {
                let group = line.split(":").next().expect(
                    "group line contains first entry",
                );
                ui.status(
                    Status::Creating,
                    format!("group '{}' in /{}", group, &file),
                )?;
                f.write_all(line.as_bytes())?;
            }
        }
        Ok(())
    }

    fn create_entrypoint(&self, ui: &mut UI) -> Result<()> {
        ui.status(Status::Creating, "entrypoint script")?;
        let ctx = self.0.ctx();
        let busybox_shell = util::pkg_path_for(&util::busybox_ident()?, ctx.rootfs())?
            .join("bin/sh");
        let json = json!({
            "busybox_shell": busybox_shell,
            "path": ctx.env_path(),
            "sup_bin": format!("{} sup", ctx.bin_path().join("hab").display()),
            "primary_svc_ident": ctx.primary_svc_ident().to_string(),
        });
        let init = ctx.rootfs().join("init.sh");
        util::write_file(&init, &Handlebars::new().template_render(INIT_SH, &json)?)?;
        filesystem::chmod(init.to_string_lossy().as_ref(), 0o0755)?;
        Ok(())
    }



}