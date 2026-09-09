# M8 — fechamento do fluxo de entrega

Especificação: `docs/superpowers/specs/2026-09-09-linux-provider-parity-and-release-design.md`.
Plano original: `docs/superpowers/plans/2026-09-09-m8-mvp-release-hardening.md`.
Base desta implementação: `1732ea09b609f03f75119676cd1e704496644347`.

## Decisões de implementação

O escopo de fechamento foi autorizado pelo mantenedor: release por tag, documentação, recuperação dos planos e aceite manual explícito. Não altera providers, interface, Cargo.lock ou comportamento do aplicativo.

O empacotamento e a inspeção dos helpers já existem no CI. Em vez de duplicar seus comandos em outro workflow ou mover verificações que já funcionam, `ci.yml` passa a ser reutilizável por `workflow_call`. O release chama exatamente esse pipeline do commit marcado.

A criação da release fica em modo **draft**. Esse é o mecanismo para respeitar o requisito de testar os pacotes em Windows/Linux X11 antes da primeira publicação. A aprovação manual é um procedimento do mantenedor, não uma certificação produzida pelo workflow.

Os quatro planos M5–M8 são recuperados sem reescrita a partir da branch `docs/m5-m8-provider-parity-design`, commit `00158a4ffc2326a5dea47d2e6e03342a7fbffc44`. Seus checkboxes históricos não são prova de status; este registro e as evidências da versão descrevem a situação efetiva.

## Verificação desta alteração

A suíte CLI foi executada antes da implementação: 17 testes falharam pela ausência de `release_tools.py`. Depois da implementação, os mesmos 17 testes passaram, usando arquivos temporários e processos reais, sem rede/provedores. Foram verificados identidade da versão, rejeição de evento de branch, configurações inválidas, formatos ausentes/duplicados, arquivos vazios, links simbólicos, nomes inseguros, hashes e preservação de arquivos existentes.

Os dois YAMLs foram analisados localmente, com checagem estrutural de trigger, dependências, permissões, reutilização do CI e publicação como draft. Isso não equivale à execução de um workflow de tag.

O ambiente local desta alteração não dispõe de Rust, Windows ou sessão gráfica Linux X11. Compilação/testes Rust e pacotes precisam de evidência do CI **deste PR**, não de uma execução antiga da main. Consulte os checks associados ao commit final do PR.

## Portões ainda necessários

- CI completo do commit final da alteração aprovado.
- PR revisado e incorporado à main.
- Workflow acionado por uma tag real, coerente com a versão, gerando os três pacotes no rascunho.
- `docs/SMOKE_TESTS.md` executado nos ambientes requeridos e evidências revisadas.
- Autorização explícita do responsável antes de publicar o rascunho.

Nenhuma tag, release pública, aprovação manual ou teste de desktop é declarado como realizado por este registro. Não excluir a branch de documentação antes de sua incorporação à main.
