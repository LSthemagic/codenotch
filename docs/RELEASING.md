# Preparação e publicação de uma versão

## O que a automação faz

`.github/workflows/release.yml` responde a tags `v*`, valida a identidade da versão e chama o próprio `ci.yml` por `workflow_call`. O build usa o commit do evento/tag; não troca para a `main` mais recente.

A publicação exige sucesso de todos os jobs reutilizados: testes das ferramentas de release, Windows e Linux. Os jobs de plataforma compilam/testam e verificam o helper nos pacotes. Em seguida, `release_tools.py` exige exatamente um `.exe`, um `.deb` e um `.AppImage`, rejeita arquivos vazios ou nomes inseguros, prepara uma pasta limpa e gera `SHA256SUMS`.

O job com `contents: write` cria apenas um **rascunho de GitHub Release**. Os jobs de validação/build têm `contents: read`. PRs normais não criam releases. Não há credenciais de assinatura fictícias nem publicação automática como versão estável.

## Antes da tag

O mantenedor deve revisar/mergear o PR de entrega com CI verde. Não crie tags falsas para testar a publicação. Não renomeie, mova ou reaproveite uma tag já publicada.

A versão da aplicação deve coincidir em `nyrva/Cargo.toml` e `nyrva/tauri.conf.json`. A tag deve ser `v` seguida dessa versão exata. No estado inicial deste fluxo, ambos declaram `0.3.0`; uma mudança de versão exige PR próprio e atualização correspondente do `Cargo.lock`. Não publique `v1.0.0` enquanto os arquivos ainda declaram `0.3.0`.

Faça uma pré-validação sem escrever no GitHub. Na raiz, em um shell com Python 3.11+:

```bash
python -m unittest discover -s tests -p 'test_release_tools.py' -v
python scripts/release_tools.py validate-tag --ref-type tag --tag v0.3.0
```

O segundo comando apenas valida os arquivos locais; **não cria nem publica uma tag**. Para outra versão, use o número declarado no commit escolhido.

## Criar o candidato

Depois de escolher um commit revisado, limpo e com CI aprovado, confira seu SHA completo e crie a tag correspondente. Para a versão `0.3.0`, os comandos abaixo são ações reais e devem ser executados deliberadamente pelo mantenedor, apenas quando essa versão for a escolhida:

```bash
git status --short
git rev-parse HEAD
git tag -a v0.3.0 -m "Nyrva 0.3.0 release candidate"
git push origin v0.3.0
```

Não execute a sequência com arquivos locais pendentes ou em um commit não aprovado. Use `git checkout <SHA_APROVADO>` antes, caso o checkout atual não seja o commit selecionado.

Acompanhe o workflow **Release** e confira a existência de todos os pacotes no rascunho. A tag cria um candidato técnico; não é o aceite do MVP. O `gh release create` usa `--verify-tag --draft` e não substitui uma release existente.

## Aceite antes da publicação

Baixe os arquivos do rascunho usando uma conta com acesso e execute [SMOKE_TESTS.md](SMOKE_TESTS.md). Registre a tag, SHA do build, execução, hashes, ambientes, resultados e responsável em um PR de evidências.

No Linux, com todos os pacotes e `SHA256SUMS` na mesma pasta:

```bash
sha256sum -c SHA256SUMS
```

No PowerShell, calcule o hash do arquivo baixado e compare com a entrada exata do manifesto:

```powershell
Get-FileHash ./NOME_DO_INSTALADOR.exe -Algorithm SHA256
```

SHA-256 confirma integridade, não assinatura/procedência por si só. Os pacotes deste fluxo não são assinados. Não desative controles de segurança para instalá-los.

Apenas depois do aceite, publique o rascunho na interface do GitHub ou com o GitHub CLI autenticado:

```bash
gh release edit v0.3.0 --draft=false
```

Use a versão realmente testada. Confira nas notas o commit, ambientes validados, limitações e vínculo para o registro de aceite. `--prerelease` é apropriado quando a versão e o anúncio forem explicitamente de pré-lançamento; não apresente esse resultado como versão estável validada.

## Falhas e recuperação

Se versão, testes ou pacotes falharem, corrija antes de tentar publicar. Não use artefatos de outro commit para completar uma execução incompleta.

Se a criação do rascunho falhar após anexar apenas parte dos arquivos, revise o rascunho existente. A automação não o sobrescreve. Um mantenedor pode remover **somente o rascunho incompleto**, preservando a tag, e reexecutar o workflow do mesmo commit. Nunca apague uma release publicada ou sua tag para reaproveitar o número de versão.

Uma mudança de código depois da tag exige uma nova versão/tag e novo aceite dos novos binários. Não substitua silenciosamente um pacote já testado ou distribuído.

## Fora deste fechamento

Renomear o repositório é uma operação administrativa separada. Até essa mudança acontecer, use o endereço real do repositório; os workflows usam `github.repository`, sem fixar um futuro nome. Limpe branches antigas somente após preservar os planos e verificar a integração na `main`.

Não são requisitos deste MVP: Wayland, macOS, nova interface, provedores novos, sincronização, telemetria e atualização automática.

## Referências

- [Workflow reutilizável do projeto](../.github/workflows/ci.yml)
- [GitHub: reutilizar workflows](https://docs.github.com/en/actions/how-tos/reuse-automations/reuse-workflows)
- [GitHub CLI: criar releases](https://cli.github.com/manual/gh_release_create)
- [Especificação de aceite M5–M8](superpowers/specs/2026-09-09-linux-provider-parity-and-release-design.md)
