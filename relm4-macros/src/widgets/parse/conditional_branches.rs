use syn::parse::ParseStream;
use syn::{Expr, Token};

use crate::widgets::{parse_util, ConditionalBranches, IfBranch, MatchArm, ParseError};

impl ConditionalBranches {
    pub(super) fn parse_if(input: ParseStream<'_>) -> Result<Self, ParseError> {
        let mut if_branches = Vec::new();
        let mut index = 0_usize;
        while input.peek(Token![if]) || input.peek(Token![else]) {
            if_branches.push(IfBranch::parse(input, index)?);
            index += 1;
        }
        Ok(Self::If(if_branches))
    }

    pub(super) fn parse_match(input: ParseStream<'_>) -> Result<Self, ParseError> {
        let match_token = input.parse()?;
        let expr = Box::new(Expr::parse_without_eager_brace(input)?);

        let braced = parse_util::braces(input)?;

        let mut match_arms = Vec::new();
        let mut index = 0_usize;
        while !braced.is_empty() {
            match_arms.push(MatchArm::parse(&braced, index)?);
            index += 1;
        }
        Ok(Self::Match((match_token, expr, match_arms)))
    }
}
