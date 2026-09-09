# Aceite manual do MVP — Windows e Linux X11

**Situação inicial: NÃO EXECUTADO. A primeira release pública está bloqueada.**

Este documento é o roteiro de validação, não evidência de aprovação. Um CI verde, um pacote gerado ou um PR mergeado não preenchem nenhuma célula automaticamente.

## Como registrar

Teste os três pacotes da mesma execução de release: NSIS no Windows, `.deb` e AppImage em Linux X11. Use pelo menos um ambiente Windows suportado e uma sessão Linux X11 real, registrando versões e limitações. Para afirmar suporte específico a outra distribuição, valide-a também.

Copie a ficha e a tabela para `docs/validation/<tag>-smoke.md` em um PR de evidências. Registre o commit completo testado, o link da execução, os nomes e SHA-256 dos pacotes. Se as evidências forem commitadas depois do build, mantenha o SHA do **build testado**, não o SHA do commit de documentação.

Resultados permitidos: **PASSOU**, **FALHOU**, **BLOQUEADO**, **NÃO EXECUTADO**. Só use **NÃO APLICÁVEL** com justificativa aprovada; falta de uma conta/provedor é BLOQUEADO, não aprovação. Não anexe segredos, transcrições privadas nem bancos dos provedores.

## Ficha por ambiente

| Campo | Registro inicial |
|---|---|
| Responsável e data/hora | NÃO EXECUTADO |
| Tag, commit completo e execução de origem | NÃO EXECUTADO |
| Nome e SHA-256 de cada pacote | NÃO EXECUTADO |
| Sistema operacional, versão e arquitetura | NÃO EXECUTADO |
| Desktop/gerenciador de janelas e tipo de sessão Linux | NÃO EXECUTADO |
| Monitores, resolução e escala/DPI | NÃO EXECUTADO |
| Versões dos provedores e condições das contas, sem segredos | NÃO EXECUTADO |
| Evidências sanitizadas e defeitos encontrados | NÃO EXECUTADO |

## Casos obrigatórios

Execute com usuário comum, a partir dos arquivos instalados. Testes que alteram hook/autostart devem ser reversíveis e preservar configurações preexistentes.

| ID | Ação e resultado esperado | Windows NSIS | Linux deb | Linux AppImage |
|---|---|---|---|---|
| S01 | Instalar/executar o pacote; a janela aparece sem crash nem erro de biblioteca ausente. | NÃO EXECUTADO | NÃO EXECUTADO | NÃO EXECUTADO |
| S02 | Conferir borda da tela e escala; arrastar, fechar e reabrir; posição persistida e acessível. | NÃO EXECUTADO | NÃO EXECUTADO | NÃO EXECUTADO |
| S03 | Interagir com bandeja, menu e janela sem roubo indevido de foco; links abrem o destino correto. | NÃO EXECUTADO | NÃO EXECUTADO | NÃO EXECUTADO |
| S04 | Ativar início automático, terminar/iniciar sessão e conferir execução; desativar e repetir. | NÃO EXECUTADO | NÃO EXECUTADO | NÃO EXECUTADO |
| S05 | Instalar hook Claude, executar uma sessão e conferir atividade; fechar Nyrva e verificar relançamento; remover hook preservando outras configurações. | NÃO EXECUTADO | NÃO EXECUTADO | NÃO EXECUTADO |
| S06 | Codex autenticado: conferir uso/atividade com raiz padrão e `CODEX_HOME` alternativo, sem alteração de credenciais. | NÃO EXECUTADO | NÃO EXECUTADO | NÃO EXECUTADO |
| S07 | Cursor autenticado: conferir uso/atividade com IDE aberta e fechada, leitura do banco sem escrita nem bloqueio provocado pela Nyrva. | NÃO EXECUTADO | NÃO EXECUTADO | NÃO EXECUTADO |
| S08 | Antigravity aberto: conferir conexão local; fechado: conferir dados antigos/alternativa sem inventar quota atual; testar ausência/bloqueio do chaveiro quando aplicável. | NÃO EXECUTADO | NÃO EXECUTADO | NÃO EXECUTADO |
| S09 | Provedor ausente/deslogado e rede indisponível: erro isolado; demais provedores e interface continuam utilizáveis. Não apague credenciais para simular a falha. | NÃO EXECUTADO | NÃO EXECUTADO | NÃO EXECUTADO |
| S10 | Revisar diagnósticos, logs e estado persistido gerados no teste: nenhum token, segredo CSRF ou conteúdo de credenciais exposto. | NÃO EXECUTADO | NÃO EXECUTADO | NÃO EXECUTADO |
| S11 | Fechar e reabrir repetidamente; sem processos inesperados ou perda de configuração. | NÃO EXECUTADO | NÃO EXECUTADO | NÃO EXECUTADO |
| S12 | Remover hook/autostart antes de desinstalar ou remover o AppImage; sem referência quebrada e sem remover dados de autenticação dos provedores. | NÃO EXECUTADO | NÃO EXECUTADO | NÃO EXECUTADO |

Em S02, teste múltiplos monitores quando disponíveis e registre a cobertura real; não marque como testada uma configuração que não foi usada. Em S08, compare com o estado real do provedor e registre qual fonte foi utilizada.

## Defeitos e reteste

Cada falha deve registrar ID do caso, ambiente, passos, resultado esperado/observado e evidência sanitizada. Corrija em PR separado ou no PR em revisão, gere novos pacotes e repita os casos afetados e a regressão necessária. Não reutilize o aceite de um binário antigo para aprovar um novo.

## Decisão de publicação

- [ ] CI e empacotamento do commit/tag aprovados; os três pacotes e seus hashes estão identificados.
- [ ] Casos obrigatórios executados e aprovados no Windows e Linux X11, sem bloqueios de provedor ocultos.
- [ ] Nenhuma falha bloqueadora aberta; limitações restantes foram documentadas e aceitas explicitamente.
- [ ] PR de evidências revisado por responsável identificado.
- [ ] Responsável conferiu se os arquivos anexados à release são os mesmos arquivos testados.
- [ ] Publicação autorizada com responsável e data registrados.

**Decisão atual: NÃO AUTORIZADA — falta execução manual.** A autorização futura deve constar da ficha da versão testada; não altere este modelo para fingir que uma execução ocorreu.
