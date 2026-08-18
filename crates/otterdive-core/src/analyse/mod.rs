mod engine;
mod extended;
mod html;
mod matcher;
mod model;
mod profile;
mod rtf;

pub use engine::{
    AnalyseEngine, AnalyseRunError, AnalyseRunInput, AnalyseRunOutput, COMPATIBILITY_STYLE_LIMIT,
    merge_pattern_results, resolve_line_styles,
};
pub use extended::{ExtendedError, translate_analyse_extended};
pub use html::write_result_html;
pub use matcher::{
    AnalyseMatcherBackend, AnalyseMatcherRouter, CancellationToken, CompiledAnalyseMatcher,
    FunctionalRegexBackend, LineIndex,
};
pub use model::{
    AnalysePattern, AnalysePatternError, AnalysePatternErrorKind, AnalyseResult, AnalyseSearchType,
    AnalyseSelection, EffectiveStyle, LinePatternMatch, LineSpan, MergedLineResult, MonacoRangeDto,
    PatternChange, PatternId, PatternResult, ProfileLoadMode, RawMatch, RgbColor, StyledSegment,
};
pub use profile::{ParsedProfile, ProfileError, apply_profile, parse_profile, write_profile};
pub use rtf::write_result_rtf;
