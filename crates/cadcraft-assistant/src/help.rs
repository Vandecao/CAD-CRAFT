// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Help: where each command is and how to do common tasks.
//!
//! Two sources: the command catalogue that the application provides (name,
//! description, ribbon tab, group, shortcut, keywords — the same data as
//! the ribbon and the command search) and the task topics below. Search is
//! local and deterministic; the model only phrases the answer.

/// One command of the application, as shown in the ribbon.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommandInfo {
    pub name: String,
    pub description: String,
    pub tab: String,
    pub group: String,
    pub shortcut: Option<String>,
    pub keywords: String,
}

/// One help topic: how to do a task.
#[derive(Clone, Copy, Debug)]
pub struct Topic {
    pub title: &'static str,
    pub keywords: &'static str,
    pub text: &'static str,
}

/// Task topics (pt-BR, the language of the interface).
pub const TOPICS: &[Topic] = &[
    Topic {
        title: "Criar um projeto",
        keywords: "novo projeto criar começar iniciar",
        text: "Arquivo > Novo cria um projeto em memória com as molduras de fábrica. Depois crie um documento e páginas (Página Inicial ou navegador Páginas), insira símbolos pelo Centro de inserção e salve com Ctrl+S como .cadcraft.",
    },
    Topic {
        title: "Salvar e abrir",
        keywords: "salvar abrir arquivo cadcraft recuperação",
        text: "Ctrl+S salva o projeto .cadcraft; Arquivo > Abrir abre. Enquanto houver alterações não salvas, o programa guarda uma cópia de recuperação, oferecida na próxima abertura se ele fechar sem salvar.",
    },
    Topic {
        title: "Inserir símbolos e dispositivos",
        keywords: "inserir símbolo dispositivo bobina contato centro de inserção",
        text: "Escolha o símbolo no Centro de inserção e clique na folha. Cada ocorrência pertence a um dispositivo (designação como -K1); funções do mesmo dispositivo podem estar em folhas diferentes, com referência cruzada.",
    },
    Topic {
        title: "Desenhar ligações",
        keywords: "ligação fio conexão desenhar rota junção",
        text: "Use a ferramenta de conexão: clique numa porta, nos pontos intermediários e na outra porta. Espaço alterna o L; Esc cancela. Ligações lógicas formam as redes; condutores físicos ficam no editor físico.",
    },
    Topic {
        title: "Numerar dispositivos, fios e páginas",
        keywords: "numerar numeração fios dispositivos páginas renumerar travado",
        text: "Dispositivos > Numerar dispositivos, Conexões > Numerar conexões e o comando Numerar páginas mostram uma prévia antes de aplicar. Números travados não mudam. Tudo pode ser desfeito com Ctrl+Z.",
    },
    Topic {
        title: "Referências cruzadas e interrupções",
        keywords: "referência cruzada interrupção potencial marcador continuação",
        text: "Pontos de interrupção com o mesmo nome continuam o potencial em outra folha; a referência mostra a página e a coluna. Inserir > Ponto de interrupção.",
    },
    Topic {
        title: "Molduras e zonas",
        keywords: "moldura carimbo zonas colunas formulário folha",
        text: "Cada página usa uma moldura com zonas (colunas e linhas) usadas nas referências. Dados-mestre > Editar moldura abre o editor; molduras podem ser importadas de EPLAN (.fn1), QElectroTech (.titleblock) e DXF.",
    },
    Topic {
        title: "Bornes, cabos e conectores",
        keywords: "borne régua cabo veia conector ponte físico",
        text: "O editor físico (Dispositivos > Bornes, Cabos) cadastra réguas, pontes, cabos, veias e conectores. Ligações físicas são separadas das lógicas.",
    },
    Topic {
        title: "Relatórios",
        keywords: "relatório lista materiais dispositivos bornes cabos índice csv xlsx",
        text: "Ferramentas > Gerar cria páginas de relatório (índice, dispositivos, materiais, ligações, bornes, cabos) com prévia; Atualizar refaz depois de mudanças. As listas também exportam em CSV e XLSX.",
    },
    Topic {
        title: "Verificações",
        keywords: "verificar verificação erros avisos consistência F8",
        text: "Ferramentas > Verificar projeto lista erros, avisos e informações com código VER. Duplo clique leva até o item; F8 e Shift+F8 navegam; uma mensagem pode ser ignorada com justificativa.",
    },
    Topic {
        title: "Importar do QElectroTech",
        keywords: "qelectrotech qet importar projeto elmt titleblock",
        text: "Arquivo > Importar > Projeto QElectroTech: Procurar, Analisar (prévia e relatório) e Criar projeto importado. Elementos .elmt e carimbos .titleblock também podem ser importados.",
    },
    Topic {
        title: "Importar do EPLAN",
        keywords: "eplan edz fn1 fabricante peças macros importar",
        text: "Dados-mestre > Importar .edz lê pacotes de fabricantes (peças e macros 2D) para as bibliotecas, com prévia e escolha em conflitos. Molduras .fn1 entram por Importar de outro sistema.",
    },
    Topic {
        title: "Exportar PDF, DXF e DWG",
        keywords: "exportar pdf dxf dwg imprimir",
        text: "Arquivo > Exportar PDF gera o PDF vetorial com links e marcadores; Exportar DXF gera um arquivo por página; DWG usa o ODA File Converter instalado pelo usuário.",
    },
    Topic {
        title: "Macros",
        keywords: "macro reutilizar biblioteca inserir macro",
        text: "Inserir > Navegador de macros mostra a biblioteca; uma macro é inserida com IDs novos e pode ser desfeita. Crie macros a partir da seleção ou da página ativa.",
    },
    Topic {
        title: "Desfazer e refazer",
        keywords: "desfazer refazer ctrl+z ctrl+y voltar",
        text: "Ctrl+Z desfaz e Ctrl+Y refaz qualquer edição, inclusive propostas do assistente aplicadas.",
    },
];

fn words(text: &str) -> Vec<String> {
    text.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.chars().count() >= 3)
        .map(str::to_owned)
        .collect()
}

fn score(query: &[String], haystack: &str) -> usize {
    let hay = haystack.to_lowercase();
    query.iter().filter(|w| hay.contains(w.as_str())).count()
}

/// Commands that best match the question, most relevant first.
pub fn find_commands<'a>(
    catalog: &'a [CommandInfo],
    query: &str,
    limit: usize,
) -> Vec<&'a CommandInfo> {
    let q = words(query);
    let mut scored: Vec<_> = catalog
        .iter()
        .map(|c| {
            let name = score(&q, &c.name) * 3;
            let rest = score(&q, &format!("{} {} {}", c.description, c.keywords, c.group));
            (name + rest, c)
        })
        .filter(|(s, _)| *s > 0)
        .collect();
    scored.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.name.cmp(&b.1.name)));
    scored.into_iter().take(limit).map(|(_, c)| c).collect()
}

/// Task topics that best match the question.
pub fn find_topics(query: &str, limit: usize) -> Vec<&'static Topic> {
    let q = words(query);
    let mut scored: Vec<_> = TOPICS
        .iter()
        .map(|t| {
            (
                score(&q, t.title) * 3 + score(&q, t.keywords) * 2 + score(&q, t.text),
                t,
            )
        })
        .filter(|(s, _)| *s > 0)
        .collect();
    scored.sort_by_key(|(s, _)| std::cmp::Reverse(*s));
    scored.into_iter().take(limit).map(|(_, t)| t).collect()
}

/// Exact command by name (case-insensitive), for "mostrar_comando".
pub fn command_by_name<'a>(catalog: &'a [CommandInfo], name: &str) -> Option<&'a CommandInfo> {
    let n = name.trim().to_lowercase();
    catalog.iter().find(|c| c.name.to_lowercase() == n)
}
