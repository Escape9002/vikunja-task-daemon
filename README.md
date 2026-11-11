> **⚠️ WARNING:** This entire project (code, architecture, and documentation) was generated using AI. Please review the codebase before deploying it to a production environment.

# 🤖 Vikunja Task Automation Daemon

The `vikunja-task-daemon` is a lightweight, long-running background service written in Rust designed to supercharge recurring tasks in [Vikunja](https://vikunja.io/).

## 💡 What it does

While Vikunja has native recurring tasks, this daemon provides advanced workflow automation that is difficult to achieve with static recurrence alone:

* **Dynamic Templating:** Injects chronological variables into task titles (e.g., automatically generating tasks named `KW46 - Weekly Report`, `KW47 - Weekly Report`).
* **Intelligent Backlog Migration:** Cleans up your boards automatically. If a recurring task is overdue and incomplete, it is moved out of your main view and into a designated "Backlog" project.
* **Pipeline Pre-generation:** Maintains a specified buffer of future tasks (e.g., always keep the next 2 weeks of instances generated in advance).
* **Idempotent Execution:** Safely checks current state before acting. If you restart the daemon, it won't create duplicate tasks.

## ⚙️ How it does it

The system is built in Rust using the asynchronous `tokio` runtime and interacts strictly with the **Vikunja v2 REST API**.

* **Stateless Loop:** Every `CHECK_INTERVAL_SECS`, the daemon queries Vikunja, computes the time deltas using local timezones (handling Daylight Saving Time safely), and performs lifecycle actions via HTTP requests.
* **Smart Updates:** It uses v2's `PATCH` endpoint with JSON Merge Patch payloads to move tasks without wiping out existing descriptions or labels.
* **Live Config Reloading:** It uses `notify-debouncer-mini` to monitor your mounted `config.yaml`. Any valid changes to the configuration are parsed and applied instantly in memory without dropping the process or restarting the container.

---

## 🚀 Getting Up and Running

The easiest way to run the daemon is via Docker Compose. The provided `Dockerfile` compiles the Rust binary statically using Musl, resulting in a tiny, secure alpine/scratch image.

1. **Create your configuration file** (`config.yaml`):
    ```yaml
    tasks:
      Operations:
        EMF:
          backlog_project_name: "Backlog"
          repetition: "1 week"
          due_day: "wednesday"
          due_time: "15:37"
          importance: "HIGH"
          name_template: "{calender_week} - EMF"
          pre_generation: 1
    ```

2. **Create your `docker-compose.yml`**:
    ```yaml
    version: "3.8"

    services:
      vikunja-daemon:
        build: .
        # Or use image: your-registry/vikunja-task-daemon:latest
        container_name: vikunja-task-daemon
        restart: unless-stopped
        environment:
          - VIKUNJA_API_URL=https://tasks.yourdomain.com/api/v2
          - VIKUNJA_API_TOKEN=your_secure_api_token
          - TIMEZONE=Europe/Berlin
          - CHECK_INTERVAL_SECS=60
          - RUST_LOG=info
        volumes:
          - ./config.yaml:/etc/vikunja-daemon/config.yaml:ro
    ```

3. **Start the container**:
    ```bash
    docker-compose up -d --build
    ```

---

## 🛠️ How to Start Developing

If you want to contribute, modify the logic, or compile the project locally, follow these steps:

### Prerequisites
* Install Rust via [rustup](https://rustup.rs/) (version 1.80+ recommended).
* Access to a Vikunja instance for testing (or run Vikunja locally via Docker).
* Generate a Vikunja API Token via *Settings > API Tokens*.

### Running Locally

1. **Clone the repository** and navigate to the root:
    ```bash
    git clone https://github.com/your-org/vikunja-task-daemon.git
    cd vikunja-task-daemon
    ```

2. **Set your environment variables**. You can export them in your terminal:
    ```bash
    export VIKUNJA_API_URL="https://tasks.example.com/api/v2"
    export VIKUNJA_API_TOKEN="your_token_here"
    export CONFIG_FILE_PATH="./config.yaml" # Point to a local test config
    export TIMEZONE="Europe/Berlin"
    export RUST_LOG="debug" # Enable verbose output for development
    ```

3. **Run the project**:
    ```bash
    cargo run
    ```
    The daemon will start up, validate the local `config.yaml`, and begin its evaluation loop against your target Vikunja instance.

4. **Useful Cargo Commands**:
    * `cargo check`: Quickly verify if your code compiles.
    * `cargo fmt`: Auto-format the code to Rust standards.
    * `cargo clippy`: Run the Rust linter to catch common mistakes or unoptimized code.

---

## 📖 Further Reading

For a deep dive into the internal algorithms, API mapping, UML system architecture, and calling-flow sequence diagrams, please refer to the project documentation:

👉 **[View the Project Architecture & Documentation](DOCUMENTATION.md)**