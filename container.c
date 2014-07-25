#include <stdio.h>
#include <string.h>
#include <sched.h>
#include <stdlib.h>
#include <unistd.h>
#include <signal.h>
#include <sys/types.h>
#include <sys/mount.h>
#include <unistd.h>
#include <errno.h>

/* Keep in sync with linux.rs */
struct mount {
    char *source;
    char *target;
    char *fstype;
    char *options;
    unsigned long flags;
};

/* Keep in sync with linux.rs */
struct container {
    int pipe_reader;
    int pipe_writer;
    const char *container_root;
    const char *mount_dir;
    int mounts_num;
    struct mount *mounts;
    const char *work_dir;
    char *exec_filename;  // Basically for error message
    int exec_filenames_num;
    char **exec_filenames;  // Real paths to try
    char **exec_args;
    char **exec_environ;
};


inline void check_error(int rc, const char *pattern, const char *val) {
    if(rc < 0) {
        fprintf(stderr, pattern, val, errno, strerror(errno));
        exit(121);
    }
}


void mount_all(int num, struct mount *mounts) {
    int i;
    for(i = 0; i < num; ++i) {
        struct mount *mnt = &mounts[i];
        if(mnt->flags & (MS_BIND | MS_RDONLY) == (MS_BIND | MS_RDONLY)) {
            //  Can bind readonly right away must first just bind
            //  then remount readonly
            int flags1 = mnt->flags & ~(MS_REMOUNT | MS_RDONLY);
            int flags2 = MS_BIND | MS_RDONLY | MS_REMOUNT;
            check_error(mount(mnt->source, mnt->target, mnt->fstype,
                              flags1, NULL),
                "Can't mount %s: (%d) %s\n", mnt->target);
            check_error(mount(mnt->source, mnt->target, mnt->fstype,
                              flags2, NULL),
                "Can't remount ro %s: (%d) %s\n", mnt->target);
        } else {
            check_error(mount(mnt->source, mnt->target, mnt->fstype,
                              mnt->flags, NULL),
                "Can't mount %s: (%d) %s\n", mnt->target);
        }
    }
}

int _run_container(void *arg) {
    int i, rc;
    char val[1];
    struct container *cont = arg;

    do {
        rc = read(cont->pipe_reader, val, 1);
    } while(rc < 0 && (errno == EINTR || errno == EAGAIN));
    check_error(rc,
        "Can't read from %s: (%d) %s\n", "pipe");
    close(cont->pipe_reader);
    close(cont->pipe_writer);

    mount_all(cont->mounts_num, cont->mounts);

    check_error(chdir(cont->mount_dir),
        "Can't set working directory to %s: (%d) %s\n", cont->mount_dir);
    check_error(chroot(cont->mount_dir),
        "Can't change root to %s: (%d) %s\n", cont->mount_dir);
    check_error(chdir(cont->work_dir),
        "Can't set working directory to %s: (%d) %s\n", cont->work_dir);

    sigset_t sigset;
    sigemptyset(&sigset);
    sigprocmask(SIG_SETMASK, &sigset, NULL);

    struct sigaction sig_action;
    sig_action.sa_handler = SIG_DFL;
    sig_action.sa_flags = 0;
    sigemptyset(&sig_action.sa_mask);

    for (i = 0 ; i < NSIG ; i++)
        sigaction(i, &sig_action, NULL);

    for(i = 0; i < cont->exec_filenames_num; ++i) {
        execve(cont->exec_filenames[i], cont->exec_args, cont->exec_environ);
    }
    check_error(-1, "Couldn't exec file %s: (%d) %s\n", cont->exec_filename);
    exit(255);
}


pid_t fork_to_container(int flags, struct container *container) {

    size_t stack_size = sysconf(_SC_PAGESIZE);
    void *stack = alloca(stack_size);

    pid_t pid = clone(_run_container,
        stack + stack_size,
        flags|SIGCHLD,
        container);

    return pid;
}
