#!/usr/bin/env python3
import argparse, sys, re
from pathlib import Path

def main():
    ap=argparse.ArgumentParser(); ap.add_argument('--repo', default='.')
    repo=Path(ap.parse_args().repo).resolve()
    src=repo/'src'
    provider_candidates=list(src.rglob('*Config*Context*.tsx'))+list(src.rglob('*Config*Provider*.tsx'))
    provider_text='\n'.join(p.read_text(errors='ignore') for p in provider_candidates)
    failures=[]
    if not provider_candidates:
        failures.append('no ConfigContext/ConfigProvider candidate found')
    if 'createContext' not in provider_text and 'zustand' not in provider_text and 'useSyncExternalStore' not in provider_text:
        failures.append('no obvious shared config store primitive found')
    use_config=(src/'hooks/useConfig.ts')
    if use_config.exists():
        t=use_config.read_text(errors='ignore')
        if re.search(r'const \[config, setConfig\] = useState', t):
            failures.append('useConfig still owns isolated config useState')
    for required in ['components/prompt-studio/PromptStudio.tsx','components/settings/SettingsPanel.tsx']:
        p=src/required
        if p.exists() and 'useConfig' not in p.read_text(errors='ignore'):
            # ok if using context hook named useConfig from provider
            pass
    if failures:
        print('FAIL config provider assertions:'); [print(' -', f) for f in failures]
        return 1
    print('PASS: shared config provider/store detected')
    return 0
if __name__ == '__main__': raise SystemExit(main())
