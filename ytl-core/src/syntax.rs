use rowan::Language;

// 1. Определяем типы узлов (Kind)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum YtlKind {
    Root,
    Whitespace,
    Comment,
    Block,
    BlockName,
    BlockParents,
    BlockBody,
    Field,
    FieldName,
    FieldValue,
    ConstKeyword,
    Include,
    Attach,
    Realm,
    Error,
}

// 2. Реализуем трейт Language для rowan
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct YtlLanguage;

impl Language for YtlLanguage {
    type Kind = YtlKind;

    fn kind_from_raw(raw: rowan::SyntaxKind) -> Self::Kind {
        // Преобразование из rowan::SyntaxKind в наш YtlKind
        match raw.0 {
            0 => YtlKind::Root,
            1 => YtlKind::Whitespace,
            2 => YtlKind::Comment,
            3 => YtlKind::Block,
            4 => YtlKind::BlockName,
            5 => YtlKind::BlockParents,
            6 => YtlKind::BlockBody,
            7 => YtlKind::Field,
            8 => YtlKind::FieldName,
            9 => YtlKind::FieldValue,
            10 => YtlKind::ConstKeyword,
            11 => YtlKind::Include,
            12 => YtlKind::Attach,
            13 => YtlKind::Realm,
            _ => YtlKind::Error,
        }
    }

    fn kind_to_raw(kind: Self::Kind) -> rowan::SyntaxKind {
        // Обратное преобразование
        rowan::SyntaxKind(kind as u16)
    }
}

// 3. Алиас для SyntaxNode, чтобы не писать везде YtlLanguage
pub type SyntaxNode = rowan::SyntaxNode<YtlLanguage>;