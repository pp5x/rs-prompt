from __future__ import annotations

import os
import re
import shlex
import shutil
import subprocess
import tempfile
from dataclasses import dataclass
from pathlib import Path

import pytest

REPO_ROOT = Path(__file__).resolve().parents[1]

RESET = "\x1b[0m"
BOLD = "\x1b[1m"
UNDERLINE = "\x1b[4m"
UNDERLINE_OFF = "\x1b[24m"
WHITE = "\x1b[37m"
YELLOW = "\x1b[33m"
RED = "\x1b[31m"
BLUE = "\x1b[34m"
GREEN = "\x1b[32m"

ANSI_RE = re.compile(r"\x1b\[[0-9;]*m")
BASH_PROMPT_MARKER_RE = re.compile(r"\\\[(.*?)\\\]")


@dataclass(frozen=True)
class Implementation:
    id: str
    root: Path
    build_command: list[str]
    binary: Path
    init_precmd: str
    init_status_assignment: str
    init_vcs_root_function: str


IMPLEMENTATIONS = [
    Implementation(
        id="rs",
        root=REPO_ROOT,
        build_command=["cargo", "build"],
        binary=REPO_ROOT / "target" / "debug" / "rs-prompt",
        init_precmd="prompt_rs_prompt_precmd",
        init_status_assignment="RS_PROMPT_STATUS=$?",
        init_vcs_root_function="__rs_prompt_outermost_vcs_root",
    ),
]


def has_vcs_marker_ancestor(path: Path) -> bool:
    resolved = path.resolve()
    for current in (resolved, *resolved.parents):
        if (
            (current / ".git").exists()
            or (current / ".jj").exists()
            or (current / ".repo").exists()
        ):
            return True
    return False


@pytest.fixture(scope="session", params=IMPLEMENTATIONS, ids=lambda impl: impl.id)
def prompt_impl(request: pytest.FixtureRequest) -> Implementation:
    implementation = request.param
    subprocess.run(
        implementation.build_command,
        cwd=implementation.root,
        check=True,
    )
    return implementation


@pytest.fixture()
def scenario_root() -> Path:
    base = Path(os.environ.get("RS_PROMPT_TEST_TMPDIR", str(Path.home())))
    if has_vcs_marker_ancestor(base):
        pytest.fail(f"test fixture base has a VCS marker ancestor: {base}")

    root = Path(tempfile.mkdtemp(prefix="prompt.", dir=base))
    try:
        yield root
    finally:
        shutil.rmtree(root)


@pytest.fixture()
def fake_home(scenario_root: Path) -> Path:
    home = scenario_root / "home"
    home.mkdir()
    return home


def run_prompt(
    implementation: Implementation,
    cwd: Path,
    home: Path,
    *,
    host: str = "host9.example",
    status: int = 0,
    shell: str | None = None,
    user: str | None = None,
    virtual_env: str | None = None,
) -> str:
    env = os.environ.copy()
    env["HOME"] = str(home)
    env["USER"] = ""
    env["LOGNAME"] = ""
    if virtual_env is None:
        env.pop("VIRTUAL_ENV", None)
    else:
        env["VIRTUAL_ENV"] = virtual_env

    command = [
        str(implementation.binary),
        "prompt",
        f"--host={host}",
        f"--cwd={cwd}",
        f"--status={status}",
    ]
    if shell is not None:
        command.append(f"--shell={shell}")
    if user is not None:
        command.append(f"--user={user}")

    result = subprocess.run(
        command,
        cwd=REPO_ROOT,
        env=env,
        check=True,
        text=True,
        capture_output=True,
    )
    return result.stdout


def strip_ansi(value: str) -> str:
    return ANSI_RE.sub("", value)


def strip_bash_prompt_markers(value: str) -> str:
    return BASH_PROMPT_MARKER_RE.sub(r"\1", value)


def q(value: Path | str) -> str:
    return shlex.quote(str(value))


def run_zsh_with_init(
    implementation: Implementation,
    body: str,
    *,
    home: Path,
) -> str:
    env = os.environ.copy()
    env["HOME"] = str(home)
    env["USER"] = ""
    env["LOGNAME"] = ""
    result = subprocess.run(
        ["zsh", "-fc", f"source <({q(implementation.binary)} init zsh)\n{body}"],
        cwd=REPO_ROOT,
        env=env,
        check=True,
        text=True,
        capture_output=True,
    )
    return result.stdout.rstrip("\n")


def run_bash_with_init(
    implementation: Implementation,
    body: str,
    *,
    home: Path,
) -> str:
    env = os.environ.copy()
    env["HOME"] = str(home)
    env["USER"] = ""
    env["LOGNAME"] = ""
    result = subprocess.run(
        [
            "bash",
            "--noprofile",
            "--norc",
            "-c",
            f"source <({q(implementation.binary)} init bash)\n{body}",
        ],
        cwd=REPO_ROOT,
        env=env,
        check=True,
        text=True,
        capture_output=True,
    )
    return result.stdout.rstrip("\n")


@pytest.fixture(scope="session")
def rs_prompt_impl() -> Implementation:
    implementation = IMPLEMENTATIONS[0]
    subprocess.run(
        implementation.build_command,
        cwd=implementation.root,
        check=True,
    )
    return implementation


def git_root(name: str) -> str:
    return f"{BOLD}{RED}{name}{RESET}{GREEN}"


def jj_root(name: str) -> str:
    return f"{BOLD}{YELLOW}{name}{RESET}{GREEN}"


def repo_root(name: str) -> str:
    return f"{BOLD}{WHITE}{name}{RESET}{GREEN}"


def prompt_end(implementation: Implementation) -> str:
    return f"{BOLD}{BLUE}%{RESET} "


def test_absolute_path_prompt_bytes(
    prompt_impl: Implementation,
    fake_home: Path,
) -> None:
    prompt = run_prompt(
        prompt_impl,
        Path("/etc/nix"),
        fake_home,
        host="host9.example",
        status=2,
    )

    assert prompt == (
        f"{WHITE}host{UNDERLINE}9{UNDERLINE_OFF}{RESET} "
        f"{GREEN}/e/nix{RESET} "
        f"[{RED}2{RESET}] {prompt_end(prompt_impl)}"
    )


def test_home_path_with_virtualenv(
    prompt_impl: Implementation,
    fake_home: Path,
) -> None:
    path = fake_home / "dev" / "foo" / "bar"
    path.mkdir(parents=True)

    prompt = run_prompt(
        prompt_impl,
        path,
        fake_home,
        host="build42.lab",
        virtual_env=str(fake_home / ".venvs" / "venv"),
    )

    assert prompt == (
        f"{WHITE}build{UNDERLINE}42{UNDERLINE_OFF}{RESET} "
        f"{YELLOW}(venv){RESET} "
        f"{GREEN}~/d/f/bar{RESET} {prompt_end(prompt_impl)}"
    )


def test_git_path_starts_at_repo_root(
    prompt_impl: Implementation,
    scenario_root: Path,
    fake_home: Path,
) -> None:
    path = scenario_root / "linux" / "drivers" / "network"
    path.mkdir(parents=True)
    (scenario_root / "linux" / ".git").mkdir()

    prompt = run_prompt(prompt_impl, path, fake_home, host="build42.lab")

    assert prompt == (
        f"{WHITE}build{UNDERLINE}42{UNDERLINE_OFF}{RESET} "
        f"{GREEN}{git_root('linux')}/d/network{RESET} {prompt_end(prompt_impl)}"
    )
    assert strip_ansi(prompt) == "build42 linux/d/network % "


def test_root_level_vcs_path_keeps_leading_slash(
    rs_prompt_impl: Implementation,
) -> None:
    with tempfile.TemporaryDirectory(dir="/tmp", prefix="rs-prompt-root-") as tmp:
        path = Path(tmp)
        (path / ".git").mkdir()

        prompt = run_prompt(rs_prompt_impl, path, Path("/home/example"))

    assert f"{GREEN}/" in prompt
    assert strip_ansi(prompt).split(" ", 2)[1].startswith("/")
    assert strip_ansi(prompt) == f"host9 /tmp/{path.name} % "


def test_nested_vcs_roots(
    prompt_impl: Implementation,
    scenario_root: Path,
    fake_home: Path,
) -> None:
    path = scenario_root / "outer" / "inner" / "deep"
    path.mkdir(parents=True)
    (scenario_root / "outer" / ".git").mkdir()
    (scenario_root / "outer" / "inner" / ".jj").mkdir()

    prompt = run_prompt(prompt_impl, path, fake_home)

    assert f"{GREEN}{git_root('outer')}/{jj_root('inner')}/deep{RESET}" in prompt
    assert strip_ansi(prompt) == "host9 outer/inner/deep % "


def test_repo_manifests_nested_git_root(
    prompt_impl: Implementation,
    scenario_root: Path,
    fake_home: Path,
) -> None:
    path = scenario_root / "aosp" / ".repo" / "manifests" / "subdir"
    path.mkdir(parents=True)
    (scenario_root / "aosp" / ".repo" / "manifests" / ".git").mkdir()

    prompt = run_prompt(prompt_impl, path, fake_home)

    assert (
        f"{GREEN}{repo_root('aosp')}/.r/{git_root('manifests')}/subdir{RESET}" in prompt
    )
    assert strip_ansi(prompt) == "host9 aosp/.r/manifests/subdir % "


def test_git_file_and_symlink_markers(
    prompt_impl: Implementation,
    scenario_root: Path,
    fake_home: Path,
) -> None:
    worktree = scenario_root / "worktree"
    worktree.mkdir()
    (worktree / ".git").write_text("gitdir: ../actual_git\n")

    target = scenario_root / "target"
    target.mkdir()
    linked = scenario_root / "linked"
    linked.mkdir()
    (linked / ".jj").symlink_to(target)

    git_prompt = run_prompt(prompt_impl, worktree, fake_home)
    jj_prompt = run_prompt(prompt_impl, linked, fake_home)

    assert f"{GREEN}{git_root('worktree')}{RESET}" in git_prompt
    assert f"{GREEN}{jj_root('linked')}{RESET}" in jj_prompt
    assert strip_ansi(git_prompt) == "host9 worktree % "
    assert strip_ansi(jj_prompt) == "host9 linked % "


def test_init_zsh_references_current_binary(prompt_impl: Implementation) -> None:
    result = subprocess.run(
        [str(prompt_impl.binary), "init", "zsh"],
        cwd=REPO_ROOT,
        check=True,
        text=True,
        capture_output=True,
    )

    assert prompt_impl.init_precmd in result.stdout
    assert prompt_impl.init_status_assignment in result.stdout
    assert f"'{prompt_impl.binary}' prompt" in result.stdout
    assert prompt_impl.init_vcs_root_function in result.stdout
    assert "--shell=zsh" in result.stdout
    assert "--prompt-escape=zsh" in result.stdout


def test_init_zsh_rendered_prompt_keeps_percent(
    prompt_impl: Implementation,
    scenario_root: Path,
    fake_home: Path,
) -> None:
    path = scenario_root / "project" / "src"
    path.mkdir(parents=True)
    (scenario_root / "project" / ".git").mkdir()

    output = run_zsh_with_init(
        prompt_impl,
        "\n".join(
            [
                "unset VIRTUAL_ENV",
                "HOST=host9.example",
                "export HOST",
                f"builtin cd {q(path)}",
                prompt_impl.init_precmd,
                'print -P -- "$PROMPT"',
            ]
        ),
        home=fake_home,
    )

    assert strip_ansi(output) == "host9 project/src % "


def test_rs_prompt_zsh_rendered_prompt_uses_prompt_escapes(
    rs_prompt_impl: Implementation,
    scenario_root: Path,
    fake_home: Path,
) -> None:
    path = scenario_root / "project" / "src"
    path.mkdir(parents=True)
    (scenario_root / "project" / ".git").mkdir()

    output = run_zsh_with_init(
        rs_prompt_impl,
        "\n".join(
            [
                "unset VIRTUAL_ENV",
                "HOST=host9.example",
                "export HOST",
                f"builtin cd {q(path)}",
                "prompt_rs_prompt_precmd",
                "__rs_prompt_render_prompt",
            ]
        ),
        home=fake_home,
    )

    assert "%{" in output
    assert "%}" in output
    assert "%%" in output


def test_rs_prompt_init_bash_references_current_binary(
    rs_prompt_impl: Implementation,
) -> None:
    result = subprocess.run(
        [str(rs_prompt_impl.binary), "init", "bash"],
        cwd=REPO_ROOT,
        check=True,
        text=True,
        capture_output=True,
    )

    assert (
        f"'{rs_prompt_impl.binary}' prompt --shell=bash --prompt-escape=bash"
        in result.stdout
    )
    assert "PROMPT_COMMAND=__rs_prompt_prompt_command" in result.stdout
    assert 'PS1="$(__rs_prompt_render_prompt)"' in result.stdout
    assert "PS1='$(__rs_prompt_render_prompt)'" not in result.stdout
    assert "RS_PROMPT_STATUS=$status" in result.stdout
    assert "__rs_prompt_outermost_vcs_root" in result.stdout


def test_rs_prompt_bash_prompt_rendering_smoke(
    rs_prompt_impl: Implementation,
    scenario_root: Path,
    fake_home: Path,
) -> None:
    path = scenario_root / "project" / "src"
    path.mkdir(parents=True)
    (scenario_root / "project" / ".git").mkdir()

    output = run_bash_with_init(
        rs_prompt_impl,
        "\n".join(
            [
                "unset VIRTUAL_ENV",
                "HOST=host9.example",
                "USER=",
                "LOGNAME=",
                "export HOST USER LOGNAME",
                f"builtin cd {q(path)}",
                "false",
                "__rs_prompt_prompt_command",
                'printf "%s" "$PS1"',
            ]
        ),
        home=fake_home,
    )

    assert "\\[" in output
    assert "\\]" in output
    assert strip_ansi(strip_bash_prompt_markers(output)) == "host9 project/src [1] $ "


def test_rs_prompt_bash_prompt_root_rendering_smoke(
    rs_prompt_impl: Implementation,
    fake_home: Path,
) -> None:
    output = run_bash_with_init(
        rs_prompt_impl,
        "\n".join(
            [
                "unset VIRTUAL_ENV",
                "HOST=host9.example",
                "USER=root",
                "export HOST USER",
                "true",
                "__rs_prompt_prompt_command",
                'printf "%s" "$PS1"',
            ]
        ),
        home=fake_home,
    )

    visible_output = strip_ansi(strip_bash_prompt_markers(output))
    assert "\\[" in output
    assert "\\]" in output
    assert visible_output.startswith("root@host9 ")
    assert visible_output.endswith("# ")


def test_rs_prompt_visible_user_and_root_cases(
    rs_prompt_impl: Implementation,
    fake_home: Path,
) -> None:
    hidden_user_prompt = run_prompt(
        rs_prompt_impl,
        Path("/etc/nix"),
        fake_home,
        user="",
    )
    visible_user_prompt = run_prompt(
        rs_prompt_impl,
        Path("/etc/nix"),
        fake_home,
        user="alice",
    )
    root_prompt = run_prompt(
        rs_prompt_impl,
        Path("/etc/nix"),
        fake_home,
        shell="bash",
        user="root",
    )

    assert strip_ansi(hidden_user_prompt).startswith("host9 ")
    assert strip_ansi(visible_user_prompt).startswith("alice@host9 ")
    assert strip_ansi(root_prompt).startswith("root@host9 ")
    assert strip_ansi(root_prompt).endswith("# ")


def test_rs_prompt_shell_specific_end_markers(
    rs_prompt_impl: Implementation,
    fake_home: Path,
) -> None:
    zsh_prompt = run_prompt(
        rs_prompt_impl,
        Path("/etc/nix"),
        fake_home,
        shell="zsh",
        user="",
    )
    bash_prompt = run_prompt(
        rs_prompt_impl,
        Path("/etc/nix"),
        fake_home,
        shell="bash",
        user="",
    )

    assert zsh_prompt.endswith(f"{BOLD}{BLUE}%{RESET} ")
    assert bash_prompt.endswith(f"{GREEN}${RESET} ")
    assert strip_ansi(bash_prompt).endswith("$ ")


def test_bare_cd_from_project_subdir_goes_to_project_root(
    prompt_impl: Implementation,
    scenario_root: Path,
    fake_home: Path,
) -> None:
    path = scenario_root / "project1" / "src" / "toto"
    path.mkdir(parents=True)
    (scenario_root / "project1" / ".git").mkdir()

    output = run_zsh_with_init(
        prompt_impl,
        f"builtin cd {q(path)}\ncd\npwd",
        home=fake_home,
    )

    assert output == str(scenario_root / "project1")


def test_bare_cd_from_nested_vcs_goes_to_innermost_root(
    prompt_impl: Implementation,
    scenario_root: Path,
    fake_home: Path,
) -> None:
    path = scenario_root / "outer" / "inner" / "src"
    path.mkdir(parents=True)
    (scenario_root / "outer" / ".repo").mkdir()
    (scenario_root / "outer" / "inner" / ".jj").mkdir()

    output = run_zsh_with_init(
        prompt_impl,
        f"builtin cd {q(path)}\ncd\npwd",
        home=fake_home,
    )

    assert output == str(scenario_root / "outer" / "inner")


def test_bare_cd_outside_vcs_goes_to_home(
    prompt_impl: Implementation,
    scenario_root: Path,
    fake_home: Path,
) -> None:
    path = scenario_root / "plain" / "src"
    path.mkdir(parents=True)

    output = run_zsh_with_init(
        prompt_impl,
        f"builtin cd {q(path)}\ncd\npwd",
        home=fake_home,
    )

    assert output == str(fake_home)


def test_explicit_cd_arguments_keep_builtin_behavior(
    prompt_impl: Implementation,
    scenario_root: Path,
    fake_home: Path,
) -> None:
    path = scenario_root / "project" / "src" / "toto"
    other = scenario_root / "other"
    path.mkdir(parents=True)
    other.mkdir()
    (scenario_root / "project" / ".git").mkdir()

    output = run_zsh_with_init(
        prompt_impl,
        "\n".join(
            [
                f"builtin cd {q(path)}",
                "cd ..",
                "pwd",
                f"cd {q(other)}",
                f"cd {q(path)}",
                "cd - >/dev/null",
                "pwd",
            ]
        ),
        home=fake_home,
    )

    assert output.splitlines() == [
        str(scenario_root / "project" / "src"),
        str(other),
    ]
