// Captain's `window.ddClient`, the v1 client of the Docker extension SDK
// (@docker/extension-api-client-types 0.4.2). Each call is posted to Captain as JSON
// with `window.ipc.postMessage`; Captain answers through `window.__captainBridge`.
// See docs/adr/0011-extensions.md.
(function () {
  "use strict";
  if (window.ddClient) {
    return;
  }
  const context = __CAPTAIN_CONTEXT__;
  const pending = new Map();
  let nextId = 1;

  function post(id, method, params) {
    window.ipc.postMessage(JSON.stringify({ id: id, method: method, params: params }));
  }

  function call(method, params) {
    return new Promise(function (resolve, reject) {
      const id = nextId++;
      pending.set(id, { resolve: resolve, reject: reject });
      post(id, method, params === undefined ? {} : params);
    });
  }

  function withHelpers(result) {
    result.lines = function () {
      return result.stdout.split(/\r?\n/);
    };
    result.parseJsonLines = function () {
      return result.lines().filter(function (line) {
        return line;
      }).map(function (line) {
        return JSON.parse(line);
      });
    };
    result.parseJsonObject = function () {
      return JSON.parse(result.stdout);
    };
    return result;
  }

  function exec(method) {
    return function (cmd, args, options) {
      if (typeof cmd !== "string") {
        throw new TypeError('The "cmd" argument must be of type string.');
      }
      if (!Array.isArray(args)) {
        throw new TypeError('The "args" argument must be an array.');
      }
      const stream = options && options.stream;
      const params = {
        cmd: cmd,
        args: args.map(String),
        cwd: options && options.cwd,
        env: options && options.env,
        stream: Boolean(stream),
      };
      if (!stream) {
        return call(method, params).then(withHelpers, function (error) {
          throw error && typeof error.stdout === "string" ? withHelpers(error) : error;
        });
      }
      const id = nextId++;
      pending.set(id, { stream: stream });
      post(id, method, params);
      return {
        close: function () {
          if (pending.delete(id)) {
            post(nextId++, "exec.close", { target: id });
          }
        },
      };
    };
  }

  function request(config) {
    const data = config.data;
    return call("extension.vm.service.request", {
      method: config.method,
      url: config.url,
      headers: config.headers || {},
      data: data === undefined || data === null || typeof data === "string" ? data : JSON.stringify(data),
    });
  }

  function send(method, url, data) {
    const headers = {};
    if (data !== undefined && data !== null && typeof data === "object") {
      headers["Content-Type"] = "application/json";
    }
    return request({ method: method, url: url, headers: headers, data: data });
  }

  function toast(level) {
    return function (message) {
      call("desktopUI.toast", { level: level, message: String(message) });
    };
  }

  window.__captainBridge = {
    resolve: function (id, value) {
      const entry = pending.get(id);
      pending.delete(id);
      if (entry && entry.resolve) {
        entry.resolve(value);
      }
    },
    reject: function (id, value) {
      const entry = pending.get(id);
      pending.delete(id);
      if (entry && entry.reject) {
        entry.reject(value);
      } else if (entry && entry.stream && entry.stream.onError) {
        entry.stream.onError(value);
      }
    },
    output: function (id, data) {
      const entry = pending.get(id);
      const onOutput = entry && entry.stream && entry.stream.onOutput;
      if (!onOutput) {
        return;
      }
      if (entry.stream.splitOutputLines) {
        onOutput(data);
      } else if (data.stdout !== undefined) {
        onOutput({ stdout: data.stdout + "\n" });
      } else {
        onOutput({ stderr: data.stderr + "\n" });
      }
    },
    exit: function (id, code) {
      const entry = pending.get(id);
      pending.delete(id);
      if (entry && entry.stream && entry.stream.onClose) {
        entry.stream.onClose(code);
      }
    },
  };

  window.ddClient = {
    extension: {
      image: context.image,
      id: context.id,
      vm: {
        cli: { exec: exec("extension.vm.cli.exec") },
        service: {
          request: request,
          get: function (url) {
            return send("GET", url);
          },
          post: function (url, data) {
            return send("POST", url, data);
          },
          put: function (url, data) {
            return send("PUT", url, data);
          },
          patch: function (url, data) {
            return send("PATCH", url, data);
          },
          delete: function (url) {
            return send("DELETE", url);
          },
          head: function (url) {
            return send("HEAD", url);
          },
        },
      },
      host: { cli: { exec: exec("extension.host.cli.exec") } },
    },
    desktopUI: {
      toast: { success: toast("success"), warning: toast("warning"), error: toast("error") },
      dialog: {
        showOpenDialog: function (options) {
          return call("desktopUI.dialog.showOpenDialog", options || {});
        },
      },
      // Not supported yet. The object exists, as in Rancher Desktop, so feature
      // checks in extensions work.
      navigate: {},
    },
    host: {
      openExternal: function (url) {
        call("host.openExternal", { url: String(url) });
      },
      platform: context.platform,
      arch: context.arch,
      hostname: context.hostname,
    },
    docker: {
      cli: { exec: exec("docker.cli.exec") },
      listContainers: function (options) {
        return call("docker.listContainers", options || {});
      },
      listImages: function (options) {
        return call("docker.listImages", options || {});
      },
    },
  };

  // @docker/docker-mui-theme reads its themes from these globals; without them an
  // extension built on it shows a blank page. `palette.docker` must be complete.
  const ramp = function (shades) {
    const keys = [100, 200, 300, 400, 500, 600, 700, 800];
    const out = {};
    keys.forEach(function (key, i) {
      out[key] = shades[i];
    });
    return out;
  };
  const docker = {
    amber: ramp(["#fff4d6", "#ffe5a3", "#ffd470", "#ffc23d", "#ff9f0a", "#d98708", "#a86806", "#784a04"]),
    blue: ramp(["#dbeeff", "#b0d8ff", "#7fbfff", "#4ca5ff", "#0a84ff", "#086fd9", "#0656a8", "#043d78"]),
    green: ramp(["#dcf7e1", "#b3edbe", "#86e298", "#5ad872", "#32d74b", "#2ab63f", "#218d31", "#176423"]),
    grey: ramp(["#f5f5f7", "#e8e8ed", "#d2d2d7", "#a1a1a8", "#6e6e76", "#3a3a3f", "#232326", "#161618"]),
    red: ramp(["#ffe3e1", "#ffc2bd", "#ff9d96", "#ff8279", "#ff6961", "#d95952", "#a84540", "#78312d"]),
    violet: ramp(["#e9e8ff", "#cfcdff", "#b1afff", "#9794ff", "#7d7aff", "#6a67d9", "#5250a8", "#3b3978"]),
  };
  const themes = {
    light: { palette: { mode: "light", docker: docker } },
    dark: { palette: { mode: "dark", docker: docker } },
  };
  window.__ddMuiV5Themes = themes;
  window.__ddMuiV6Themes = themes;
})();
