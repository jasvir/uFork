/*jslint browser */

import hex from "https://ufork.org/lib/hex.js";
import parseq from "https://ufork.org/lib/parseq.js";
import requestorize from "https://ufork.org/lib/rq/requestorize.js";
import ufork from "https://ufork.org/js/ufork.js";
import awp_device from "https://ufork.org/js/awp_device.js";
import host_device from "https://ufork.org/js/host_device.js";
import webrtc_transport from "https://ufork.org/js/webrtc_transport.js";
import websockets_signaller from "https://ufork.org/js/websockets_signaller.js";
const wasm_url = import.meta.resolve("https://ufork.org/wasm/ufork.wasm");

const signaller_origin = (
    location.protocol === "https:"
    ? "wss://"
    : "ws://"
) + location.host;
const import_map = JSON.parse(
    document.querySelector("[type=importmap]").textContent
).imports;

function party(asm_url, acquaintance_names = []) {
    const pre = document.createElement("pre");
    document.body.append(pre);

    function print(...things) {
        things = things.map(function (thing) {
            return thing?.message ?? thing;
        });
        pre.textContent += "\n" + things.join(" ");
    }

    const transport = webrtc_transport(websockets_signaller(), print);
    const core = ufork.make_core({
        wasm_url,
        on_wakeup() {
            const sig = core.h_run_loop(0);
            const err = core.u_fix_to_i32(sig);
            print("WAKE:", core.u_print(sig), core.u_fault_msg(err));
        },
        on_log(log_level, ...values) {
            print(log_level, ...values);
            if (values.includes("(+0 . #?)")) {
                const div = document.createElement("div");
                div.textContent = "💸";
                div.style.fontSize = "100px";
                document.body.append(div);
            }
        },
        log_level: ufork.LOG_DEBUG,
        import_map
    });

    return parseq.sequence([
        core.h_initialize(),
        parseq.parallel([
            core.h_import(asm_url),
            transport.generate_identity()
        ]),
        requestorize(function ([asm_module, identity]) {
            const name = transport.identity_to_name(identity);
            const address = signaller_origin;
            const make_dynamic_device = host_device(core);
            awp_device({
                core,
                make_dynamic_device,
                transport,
                stores: [{
                    identity,
                    bind_info: {
                        origin: signaller_origin,
                        password: "uFork"
                    },
                    acquaintances: [
                        {name, address},
                        ...acquaintance_names.map(function (name) {
                            return {name, address: signaller_origin};
                        })
                    ]
                }]
            });
            core.h_boot(asm_module.boot);
            const sig = core.h_run_loop(0);
            const err = core.u_fix_to_i32(sig);
            print("IDLE:", core.u_print(sig), core.u_fault_msg(err));
            return name;
        })
    ])(function callback(name, reason) {
        if (name !== undefined) {
            print("Name", hex.encode(name));
        } else {
            print(reason);
        }
    });
}

export default Object.freeze(party);
