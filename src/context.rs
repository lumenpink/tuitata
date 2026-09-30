use std::fs;
use std::path::{Path, PathBuf};

const MAX_FILE_SIZE: u64 = 512 * 1024; // 500 KB

/// Expande til (`~`) para o diretório HOME do usuário.
pub fn expand_path(p: &str) -> PathBuf {
    if p.starts_with("~/") {
        if let Ok(home) = std::env::var("HOME") {
            return PathBuf::from(home).join(p.trim_start_matches("~/"));
        }
    } else if p == "~" {
        if let Ok(home) = std::env::var("HOME") {
            return PathBuf::from(home);
        }
    }
    PathBuf::from(p)
}

/// Identifica a linguagem do código a partir da extensão do arquivo.
pub fn get_file_lang(path: &Path) -> &str {
    match path.extension().and_then(|s| s.to_str()).unwrap_or("") {
        "rs" => "rust",
        "py" => "python",
        "js" => "javascript",
        "ts" => "typescript",
        "json" => "json",
        "toml" => "toml",
        "yaml" | "yml" => "yaml",
        "sh" | "bash" => "bash",
        "md" => "markdown",
        "html" => "html",
        "css" => "css",
        "sql" => "sql",
        "c" | "h" => "c",
        "cpp" | "hpp" => "cpp",
        _ => "",
    }
}

/// Lê o conteúdo de um arquivo de texto com validação de tamanho e existência.
pub fn read_file_content(path_str: &str) -> Result<(String, String), String> {
    let path = expand_path(path_str);
    if !path.exists() {
        return Err(format!("Arquivo não encontrado: {}", path_str));
    }
    if !path.is_file() {
        return Err(format!("O caminho não é um arquivo regular: {}", path_str));
    }

    let metadata = fs::metadata(&path).map_err(|e| format!("Falha ao ler metadados de {}: {}", path_str, e))?;
    if metadata.len() > MAX_FILE_SIZE {
        return Err(format!(
            "Arquivo muito grande ({} KB). O limite máximo suportado é de 500 KB.",
            metadata.len() / 1024
        ));
    }

    let content = fs::read_to_string(&path)
        .map_err(|e| format!("Falha ao ler conteúdo do arquivo {}: {}. Verifique se é texto UTF-8.", path_str, e))?;

    let lang = get_file_lang(&path);
    let formatted = format!(
        "\n--- Contexto de Arquivo: `{}` ---\n```{}\n{}\n```\n",
        path_str, lang, content
    );

    Ok((formatted, path_str.to_string()))
}

/// Processa a entrada do usuário detectando comandos `/file`, `/ler` ou menções `@arquivo`.
pub fn process_input_files(raw_input: &str) -> Result<(String, Vec<String>), String> {
    let trimmed = raw_input.trim();

    // Caso 1: Comandos `/file <caminho> [pergunta]` ou `/ler <caminho> [pergunta]`
    if trimmed.starts_with("/file ") || trimmed.starts_with("/ler ") {
        let rest = if trimmed.starts_with("/file ") {
            trimmed.trim_start_matches("/file ").trim()
        } else {
            trimmed.trim_start_matches("/ler ").trim()
        };

        let mut parts = rest.splitn(2, ' ');
        let file_path = parts.next().unwrap_or("").trim();
        let question = parts.next().unwrap_or("Analise o conteúdo do arquivo acima.").trim();

        if file_path.is_empty() {
            return Err("Uso: /file <caminho> [pergunta]".to_string());
        }

        let (file_block, loaded_path) = read_file_content(file_path)?;
        let final_prompt = format!("{}\n---\n{}", file_block, question);
        return Ok((final_prompt, vec![loaded_path]));
    }

    // Caso 2: Sintaxe com `@arquivo` no meio do texto
    let mut attached_files = Vec::new();
    let mut file_blocks = String::new();
    let mut clean_words = Vec::new();

    for word in trimmed.split_whitespace() {
        if word.starts_with('@') && word.len() > 1 {
            let possible_path = word.trim_start_matches('@').trim_matches(|c| c == ',' || c == '.' || c == ':' || c == ';');
            let test_path = expand_path(possible_path);
            if test_path.is_file() {
                match read_file_content(possible_path) {
                    Ok((block, p)) => {
                        file_blocks.push_str(&block);
                        attached_files.push(p);
                        continue;
                    }
                    Err(e) => return Err(e),
                }
            }
        }
        clean_words.push(word);
    }

    if !attached_files.is_empty() {
        let final_prompt = format!("{}\n---\n{}", file_blocks, clean_words.join(" "));
        return Ok((final_prompt, attached_files));
    }

    Ok((raw_input.to_string(), Vec::new()))
}
