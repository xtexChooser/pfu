//! Editor wrappers for modifying LST easily.
//!
//! Editors works based on the assumption that input LSTs are valid,
//! and guarantees that the output LST is valid and the newly added parts
//! are in the conventional code style.
//!
//! Editors are not alternative to PFU. It is designed for generic usages,
//! not exposing too much about styling details.
//! It basically just allows to add, rewrite and remove existing variable
//! definitions.

use super::{
	ast::{self, AstNode},
	lst::{self, ApmlLst},
};

#[derive(Debug, Clone)]
pub struct ApmlEditor<'a>(ApmlLst<'a>);

impl<'a> AsRef<ApmlLst<'a>> for ApmlEditor<'a> {
	fn as_ref(&self) -> &ApmlLst<'a> {
		&self.0
	}
}

impl<'a> ApmlEditor<'a> {
	/// Wraps the given LST with editing API.
	pub fn wrap(lst: ApmlLst<'a>) -> Self {
		Self(lst)
	}

	/// Unwraps the LST from the editing API.
	pub fn unwrap(self) -> ApmlLst<'a> {
		self.0
	}
}

impl<'a> ApmlEditor<'a> {
	/// Returns a [Vec] including all LST tokens.
	pub fn lst_tokens(&mut self) -> &Vec<lst::Token<'a>> {
		&self.0.0
	}

	/// Iterates over all LST tokens.
	pub fn lst_tokens_iter(&self) -> impl Iterator<Item = &lst::Token<'a>> {
		self.0.0.iter()
	}

	/// Returns a [Vec] including all LST tokens.
	pub fn lst_tokens_mut(&mut self) -> &mut Vec<lst::Token<'a>> {
		&mut self.0.0
	}

	/// Iterates over all variable definitions in LST form.
	pub fn lst_variables(
		&self,
	) -> impl Iterator<Item = &lst::VariableDefinition<'a>> {
		self.lst_tokens_iter().filter_map(|token| {
			if let lst::Token::Variable(var) = token {
				Some(var)
			} else {
				None
			}
		})
	}

	/// Iterates over all variables definitions in AST form.
	pub fn ast_variables(
		&self,
	) -> ast::EmitResult<Vec<ast::VariableDefinition<'a>>> {
		self.lst_variables()
			.map(ast::VariableDefinition::emit_from)
			.collect()
	}

	/// Iterates over all variable definition keys.
	pub fn keys(&self) -> impl Iterator<Item = &str> {
		self.lst_variables().map(|var| var.name.as_ref())
	}

	/// Finds a variable definition and its index.
	pub fn find_var<S: AsRef<str>>(
		&self,
		name: S,
	) -> Option<(usize, &lst::VariableDefinition<'a>)> {
		self.lst_tokens_iter().zip(0..).find_map(|(token, idx)| {
			if let lst::Token::Variable(var) = token {
				if var.name.as_ref() == name.as_ref() {
					Some((idx, var))
				} else {
					None
				}
			} else {
				None
			}
		})
	}

	/// Ensures there is a newline after the text.
	pub fn ensure_end_newline(&mut self) {
		if !matches!(self.lst_tokens().last(), None | Some(lst::Token::Newline))
		{
			self.lst_tokens_mut().push(lst::Token::Newline);
		}
	}

	/// Appends a new variable assignment definition.
	pub fn append_var_ast(
		&mut self,
		name: &'a str,
		value: &ast::VariableValue<'a>,
		after: Option<&str>,
	) {
		let definition = lst::VariableDefinition {
			name: name.into(),
			op: lst::VariableOp::Assignment,
			value: value.lower(),
		};
		let token = lst::Token::Variable(definition);
		if let Some(after) = after {
			if let Some((index, _)) = self.find_var(after) {
				let after = self
					.lst_tokens_iter()
					.skip(index)
					.take_while(|token| !matches!(token, lst::Token::Newline))
					.count();
				let index = index + after + 1;
				if index <= self.lst_tokens().len() {
					self.lst_tokens_mut().insert(index, lst::Token::Newline);
					self.lst_tokens_mut().insert(index, token);
					return;
				}
			}
		}
		self.ensure_end_newline();
		self.lst_tokens_mut().push(token);
		self.lst_tokens_mut().push(lst::Token::Newline);
	}

	/// Replace a variable definition.
	pub fn replace_var_ast(
		&mut self,
		name: &'a str,
		value: &ast::VariableValue<'a>,
	) {
		let definition = lst::VariableDefinition {
			name: name.into(),
			op: lst::VariableOp::Assignment,
			value: value.lower(),
		};
		let token = lst::Token::Variable(definition);
		if let Some((index, _)) = self.find_var(name) {
			self.lst_tokens_mut()[index] = token;
			return;
		}
		self.ensure_end_newline();
		self.lst_tokens_mut().push(token);
		self.lst_tokens_mut().push(lst::Token::Newline);
	}

	/// Removes a variable definition.
	///
	/// The given index must points to a variable definition token.
	/// After a removal, all indexes are invalidated.
	///
	/// Spaces and comments before the most near following newline will
	/// be stripped. If there is comment before the line of variable definition
	/// and the variable definition has been followed by two newlines,
	/// All preceding comments will be stripped as well.
	pub fn remove_var(&mut self, index: usize) {
		// scan of following spaces and newline
		let after = self
			.lst_tokens_iter()
			.skip(index)
			.take_while(|token| !matches!(token, lst::Token::Newline))
			.count();
		let mut start = index;
		let tokens = self.lst_tokens();
		if matches!(tokens.get(index - 2), Some(lst::Token::Comment(_))) {
			// scan for next line
			if !tokens
				.iter()
				.skip(index)
				.skip_while(|token| !matches!(token, lst::Token::Newline))
				.skip(1)
				.take_while(|token| !matches!(token, lst::Token::Newline))
				.any(|token| matches!(token, lst::Token::Variable(_)))
			{
				// next line is empty, scan for removable comments
				while matches!(tokens.get(start - 1), Some(lst::Token::Newline))
					&& matches!(
						tokens.get(start - 2),
						Some(lst::Token::Comment(_))
					) && matches!(
					tokens.get(start - 3),
					Some(lst::Token::Newline)
				) {
					start -= 2;
				}
			}
		}
		self.lst_tokens_mut().drain(start..=(index + after));
	}

	/// Iterates over all comment lines.
	pub fn comments(&self) -> impl Iterator<Item = &str> {
		self.lst_tokens_iter().filter_map(|token| {
			if let lst::Token::Comment(var) = token {
				Some(var.as_ref())
			} else {
				None
			}
		})
	}
}

#[cfg(test)]
mod test {
	use crate::apml::lst::ApmlLst;

	use super::*;

	#[test]
	fn test() {
		let lst = ApmlLst::parse("a=b\nb=c\nc=\"$1\"").unwrap();
		let mut editor = ApmlEditor::wrap(lst);
		assert_eq!(editor.lst_tokens_iter().count(), 5);
		assert_eq!(editor.lst_tokens_mut().len(), 5);
		assert_eq!(editor.ast_variables().unwrap().len(), 3);
		assert_eq!(editor.keys().count(), 3);
		assert_eq!(editor.find_var("a").unwrap().0, 0);
		assert_eq!(editor.find_var("a").unwrap().1.name, "a");
		assert_eq!(editor.find_var("b").unwrap().0, 2);
		assert!(editor.find_var("A").is_none());
		let _ = editor.unwrap();
	}

	#[test]
	fn test_ensure_end_newline() {
		let mut editor = ApmlEditor::wrap(ApmlLst::parse("a=b").unwrap());
		editor.ensure_end_newline();
		assert_eq!(editor.unwrap().to_string(), "a=b\n");
	}

	#[test]
	fn test_append_variable() {
		let mut editor = ApmlEditor::wrap(ApmlLst::parse("a=b\nb=c").unwrap());
		editor.append_var_ast(
			"c",
			&ast::VariableValue::String("a".into()),
			None,
		);
		assert_eq!(editor.unwrap().to_string(), "a=b\nb=c\nc=\"a\"\n");
		let mut editor = ApmlEditor::wrap(ApmlLst::parse("a=b\nb=c").unwrap());
		editor.append_var_ast(
			"c",
			&ast::VariableValue::String("a".into()),
			Some("a"),
		);
		assert_eq!(editor.unwrap().to_string(), "a=b\nc=\"a\"\nb=c");
		let mut editor = ApmlEditor::wrap(ApmlLst::parse("a=b\nb=c").unwrap());
		editor.append_var_ast(
			"c",
			&ast::VariableValue::String("a".into()),
			Some("b"),
		);
		assert_eq!(editor.unwrap().to_string(), "a=b\nb=c\nc=\"a\"\n");
		let mut editor = ApmlEditor::wrap(ApmlLst::parse("a=b\nb=c").unwrap());
		editor.append_var_ast(
			"c",
			&ast::VariableValue::String("a".into()),
			Some("eee"),
		);
		assert_eq!(editor.unwrap().to_string(), "a=b\nb=c\nc=\"a\"\n");
	}

	#[test]
	fn test_replace_variable() {
		let mut editor = ApmlEditor::wrap(ApmlLst::parse("a=b\nb=c").unwrap());
		editor.replace_var_ast("c", &ast::VariableValue::String("a".into()));
		assert_eq!(editor.unwrap().to_string(), "a=b\nb=c\nc=\"a\"\n");
		let mut editor = ApmlEditor::wrap(ApmlLst::parse("a=b\nb=c").unwrap());
		editor.replace_var_ast("a", &ast::VariableValue::String("a".into()));
		assert_eq!(editor.unwrap().to_string(), "a=\"a\"\nb=c");
	}

	#[test]
	fn test_remove_var() {
		let mut editor =
			ApmlEditor::wrap(ApmlLst::parse("a=b\nb=c\n\nc=\"$1\"").unwrap());
		editor.remove_var(editor.find_var("b").unwrap().0);
		assert_eq!(editor.unwrap().to_string(), "a=b\n\nc=\"$1\"");
		let mut editor = ApmlEditor::wrap(
			ApmlLst::parse("a=b # a\n# b\n# c\nb=c\n\n# a\nc=\"$1\"").unwrap(),
		);
		editor.remove_var(editor.find_var("b").unwrap().0);
		assert_eq!(editor.unwrap().to_string(), "a=b # a\n\n# a\nc=\"$1\"");
	}

	#[test]
	fn test_comments() {
		let editor = ApmlEditor::wrap(
			ApmlLst::parse("a=b # a\n# b\n# c\nb=c\n\n# a\nc=\"$1\"").unwrap(),
		);
		assert_eq!(editor.comments().count(), 4);
	}
}
