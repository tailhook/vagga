#include <sys/prctl.h>
#include <alloca.h>
#include <unistd.h>
#include <signal.h>
#include <sched.h>
#include <unistd.h>
#include <errno.h>
#include <stdio.h>
#include <stdlib.h>


typedef struct {
    int namespaces;
    int pipe_reader;
    int user_id;
    int restore_sigmask;
    const char *logprefix;
    const char *fs_root;
    const char *exec_path;
    char ** const exec_args;
    char ** const exec_environ;
    const char *workdir;
} CCommand;

typedef struct {
    int signo;
    pid_t pid;
    int status;
} CSignalInfo;

static void _run_container(CCommand *cmd) {
    prctl(PR_SET_PDEATHSIG, SIGKILL, 0, 0, 0);

    //  Wait for user namespace to be set up
    int rc;
    char val[1];
    do {
        rc = read(cmd->pipe_reader, val, 1);
    } while(rc < 0 && (errno == EINTR || errno == EAGAIN));
    if(rc < 0) {
        fprintf(stderr, "%s Error reading from parent's pipe: %m\n",
            cmd->logprefix);
        abort();
    }
    close(cmd->pipe_reader);

    if(chdir(cmd->fs_root)) {
        fprintf(stderr, "%s Error changing workdir to the root %s: %m\n",
            cmd->logprefix, cmd->fs_root);
        abort();
    }
    if(chroot(cmd->fs_root)) {
        fprintf(stderr, "%s Error changing root %s: %m\n",
            cmd->logprefix, cmd->fs_root);
        abort();
    }
    if(chdir(cmd->workdir)) {
        fprintf(stderr, "%s Error changing workdir %s: %m\n",
            cmd->logprefix, cmd->workdir);
        abort();
    }
    if(setuid(cmd->user_id)) {
        fprintf(stderr, "%s Error setting userid %d: %m\n",
            cmd->logprefix, cmd->user_id);
        abort();
    }
    if(cmd->restore_sigmask) {
        sigset_t mask;
        sigfillset(&mask);
        sigprocmask(SIG_UNBLOCK, &mask, NULL);
    }
    (void)execve(cmd->exec_path, cmd->exec_args, cmd->exec_environ);
    _exit(127);
}

pid_t execute_command(CCommand *cmd) {
    size_t stack_size = sysconf(_SC_PAGESIZE);
    void *stack = alloca(stack_size);

    return clone((int (*)(void*))_run_container,
        stack + stack_size,
        cmd->namespaces|SIGCHLD,
        cmd);
}

void block_all_signals() {
    sigset_t mask;
    sigfillset(&mask);
    sigprocmask(SIG_BLOCK, &mask, NULL);
}

int wait_any_signal(CSignalInfo *sig, struct timespec *ts) {
    sigset_t mask;
    sigfillset(&mask);
    while(1) {
        siginfo_t native_info;
        int rc;
        if(ts) {
            rc = sigtimedwait(&mask, &native_info, ts);
        } else {
            rc = sigwaitinfo(&mask, &native_info);
        }
        if(rc < 0){
            if(errno == EINTR) {
                return 1;
            } else if(errno == EAGAIN) {
                return 1;
            } else {
                fprintf(stderr, "Wrong error code for sigwaitinfo: %m\n");
                abort();
            }
        }
        sig->signo = native_info.si_signo;
        sig->pid = native_info.si_pid;
        sig->status = native_info.si_code == CLD_EXITED
            ? native_info.si_status
            : 128 + native_info.si_status;  // Wrapped signal
        return 0;
    }
}

