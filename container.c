#include <sys/prctl.h>
#include <sys/signalfd.h>
#include <sys/epoll.h>
#include <alloca.h>
#include <unistd.h>
#include <signal.h>
#include <sched.h>
#include <unistd.h>
#include <errno.h>
#include <stdio.h>
#include <stdlib.h>
#include <fcntl.h>
#include <unistd.h>


typedef struct {
    int namespaces;
    int pipe_reader;
    int user_id;
    int restore_sigmask;
    int stdin;
    int stdout;
    int stderr;
    const char *logprefix;
    const char *fs_root;
    const char *exec_path;
    char ** const exec_args;
    char ** const exec_environ;
    const char *workdir;
} CCommand;

static void _run_container(CCommand *cmd) {
    prctl(PR_SET_PDEATHSIG, SIGKILL, 0, 0, 0);

    if(cmd->stdin < 0) {
        close(0);
        if(open("/dev/null", O_RDONLY) != 0) {
            fprintf(stderr, "%s Error opening /dev/null for stdin: %m\n",
                cmd->logprefix);
            abort();
        }
    } else if(cmd->stdin > 0) {
        if(dup2(cmd->stdin, 0) < 0) {
            fprintf(stderr, "%s Error opening stdin: %m\n", cmd->logprefix);
            abort();
        }
    }
    if(dup2(cmd->stdout, 1) < 0) {
        fprintf(stderr, "%s Error setting up stdout: %m\n", cmd->logprefix);
        abort();
    }
    if(dup2(cmd->stderr, 2) < 0) {
        fprintf(stderr, "%s Error setting up stderr: %m\n", cmd->logprefix);
        abort();
    }

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

int create_signalfd() {
    sigset_t mask;
    sigfillset(&mask);
    return signalfd(-1, &mask, SFD_CLOEXEC|SFD_NONBLOCK);
}

int create_epoll() {
    return epoll_create1(EPOLL_CLOEXEC);
}

int epoll_wait_read(int epoll, int timeo) {
    struct epoll_event ev;
    int rc = epoll_wait(epoll, &ev, 1, timeo);
    if(rc > 0) {
        return ev.data.fd;
    }
    if(rc == 0)
        return -ETIMEDOUT;
    return -errno;
}

int epoll_add_read(int epoll, int fd) {
    struct epoll_event ev = {
        .events = EPOLLIN,
        .data = {
            .fd = fd,
        }
    };
    return epoll_ctl(epoll, EPOLL_CTL_ADD, fd, &ev);
}

int read_signal(int fd) {
    struct signalfd_siginfo info;
    int rc;
    rc = read(fd, &info, sizeof(info));
    if(rc > 0) {
        return info.ssi_signo;
    }
    return -errno;
}

void set_cloexec(int fd, int flag) {
    int flags = fcntl(fd, F_GETFD);
    if(flags < 0) {
        fprintf(stderr, "[vagga] Error getting flags: %m\n");
        abort();
    }
    int rc = fcntl(fd, F_SETFD, flags | FD_CLOEXEC);
    if(rc < 0) {
        fprintf(stderr, "[vagga] Error setting flags: %m\n");
        abort();
    }
}


