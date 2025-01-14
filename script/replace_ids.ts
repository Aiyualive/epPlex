#!/usr/bin/env node
/* eslint-disable @typescript-eslint/no-var-requires */
/* eslint-disable import/no-extraneous-dependencies */

/**
 * This script will
 *  - Build any anchor projects if missing
 *  - Grab anchor project IDs
 *  - Update project IDs in Anchor.toml and lib.rs
 */

import shell from "shelljs"
import {spawn, execSync} from "child_process"
import web3 from "@solana/web3.js"
import fs from "fs"
import path from "path"

const projectRoot = path.join(__dirname, "..");
const targetDir = path.join(projectRoot, "target");
const anchorToml = path.join(projectRoot, "Anchor.toml");


function replaceId(programName: string, programDir: string, idFile: string) {
    const pid = web3.Keypair.fromSecretKey(
        new Uint8Array(
            JSON.parse(fs.readFileSync(
                path.join(
                    targetDir,
                    "deploy",
                    `${programName}-keypair.json`
                ),
                "utf8"
            ))
        )
    ).publicKey;

    // REPLACE PROGRAM IDS
    console.log(`Replace Program ID:    ${pid}`);
    shell.sed(
        "-i",
        /declare_id!(.*);/,
        `declare_id!("${pid.toString()}");`,
        path.join(projectRoot, "programs", programDir, "src", idFile)
    );
}

async function main() {
    shell.cd(projectRoot);

    if (!shell.which("solana")) {
        shell.echo(
            "Sorry, this script requires 'solana' to be installed in your $PATH"
        );
        shell.exit(1);
    }

    if (!shell.which("anchor")) {
        shell.echo(
            "Sorry, this script requires 'anchor' to be installed in your $PATH"
        );
        shell.exit(1);
    }

    if (!fs.existsSync(path.join(targetDir, "deploy"))) {
        shell.echo("Missing program deploy keypairs, building projects");
        const anchorBuildSpawn = spawn("anchor", ["build"]);
        anchorBuildSpawn.stdout.on("data", function (msg) {
            console.log(msg.toString());
        });
        await new Promise((resolve) => {
            anchorBuildSpawn.on("close", resolve);
        });
    }

    replaceId("epplex_burger", "epplex-burger", "id.rs")
    replaceId("epplex_core", "epplex-core", "id.rs")
    replaceId("epplex_shared", "epplex-shared", "id.rs")
    replaceId("wen_new_standard", "wen_new_standard", "lib.rs")
    replaceId("wen_royalty_distribution", "wen_royalty_distribution", "lib.rs")

}

main()
    .then(() => {
        console.log("Executed successfully");
    })
    .catch((err) => {
        console.error(err);
    });
