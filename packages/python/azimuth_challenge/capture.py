"""Capture accountable source input files without reading ignored local secrets."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
from .scope import located
from azimuth_run.protocol import fingerprint


def capture(root, repository, git):
    root=root.resolve(strict=True)
    if not Path(git).is_absolute():
        raise ValueError('git executable must be explicitly located')
    def invoke(*arguments):
        return subprocess.check_output([git,'-C',str(root),*arguments],env={},timeout=30)
    revision=invoke('rev-parse','HEAD').decode().strip()
    paths=invoke('ls-files','--cached','--others','--exclude-standard','-z').decode().split('\0')
    files,deleted,links=[],[],[]
    for relative in sorted(set(paths)-{''}):
        path=root/relative
        if path.is_symlink():
            target=os.readlink(path)
            if Path(target).is_absolute():
                raise ValueError(f'absolute source link: {relative}')
            try:
                resolved=path.resolve(strict=False)
                resolved.relative_to(root)
            except (ValueError,OSError,RuntimeError) as error:
                raise ValueError(f'escaping source link: {relative}') from error
            links.append({'file':relative,'target':target,'resolved':resolved.relative_to(root).as_posix(),'target_exists':resolved.exists(),
                          'fingerprint':'sha256:'+hashlib.sha256(target.encode()).hexdigest()})
            continue
        if not path.exists():
            deleted.append(relative)
            continue
        data=located(root,relative).read_bytes()
        files.append({'file':relative,'fingerprint':'sha256:'+hashlib.sha256(data).hexdigest()})
    content_fingerprint=fingerprint({'revision':revision,'files':files,'deleted':deleted,'links':links})
    verification_target={'kind':'source','repositories':[{'id':repository,'revision':revision,'content_fingerprint':content_fingerprint}]}
    report={'format':'azimuth-source-capture','version':2,'repository':repository,'verification_target':verification_target,
            'files':files,'deleted':deleted,'links':links,'capture_policy':'tracked-and-untracked-not-ignored',
            'limitations':['Ignored build inputs and external dependencies are outside this capture; dependency completeness requires review.']}
    report['fingerprint']=fingerprint(report)
    return report


def main():
    p=argparse.ArgumentParser()
    p.add_argument('--root',required=True)
    p.add_argument('--repository',required=True)
    p.add_argument('--git-executable',required=True)
    p.add_argument('--out',required=True)
    a=p.parse_args()
    try:
        report=capture(Path(a.root),a.repository,a.git_executable)
        Path(a.out).write_text(json.dumps(report,indent=2)+'\n')
    except (ValueError,OSError,subprocess.SubprocessError) as error:
        p.exit(2,f'azimuth.challenge capture: {error}\n')
    return 0
if __name__=='__main__':
    raise SystemExit(main())
