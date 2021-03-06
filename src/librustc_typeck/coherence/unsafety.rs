// Copyright 2014 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Unsafety checker: every impl either implements a trait defined in this
//! crate or pertains to a type defined in this crate.

use middle::def_id::DefId;
use middle::ty;
use syntax::ast::{Item, ItemImpl};
use syntax::ast;
use syntax::visit;

pub fn check(tcx: &ty::ctxt) {
    let mut orphan = UnsafetyChecker { tcx: tcx };
    visit::walk_crate(&mut orphan, tcx.map.krate());
}

struct UnsafetyChecker<'cx, 'tcx:'cx> {
    tcx: &'cx ty::ctxt<'tcx>
}

impl<'cx, 'tcx, 'v> UnsafetyChecker<'cx, 'tcx> {
    fn check_unsafety_coherence(&mut self, item: &'v ast::Item,
                                unsafety: ast::Unsafety,
                                polarity: ast::ImplPolarity) {
        match self.tcx.impl_trait_ref(DefId::local(item.id)) {
            None => {
                // Inherent impl.
                match unsafety {
                    ast::Unsafety::Normal => { /* OK */ }
                    ast::Unsafety::Unsafe => {
                        span_err!(self.tcx.sess, item.span, E0197,
                                  "inherent impls cannot be declared as unsafe");
                    }
                }
            }

            Some(trait_ref) => {
                let trait_def = self.tcx.lookup_trait_def(trait_ref.def_id);
                match (trait_def.unsafety, unsafety, polarity) {
                    (ast::Unsafety::Unsafe,
                     ast::Unsafety::Unsafe, ast::ImplPolarity::Negative) => {
                        span_err!(self.tcx.sess, item.span, E0198,
                                  "negative implementations are not unsafe");
                    }

                    (ast::Unsafety::Normal, ast::Unsafety::Unsafe, _) => {
                        span_err!(self.tcx.sess, item.span, E0199,
                                  "implementing the trait `{}` is not unsafe",
                                  trait_ref);
                    }

                    (ast::Unsafety::Unsafe,
                     ast::Unsafety::Normal, ast::ImplPolarity::Positive) => {
                        span_err!(self.tcx.sess, item.span, E0200,
                                  "the trait `{}` requires an `unsafe impl` declaration",
                                  trait_ref);
                    }

                    (ast::Unsafety::Unsafe,
                     ast::Unsafety::Normal, ast::ImplPolarity::Negative) |
                    (ast::Unsafety::Unsafe,
                     ast::Unsafety::Unsafe, ast::ImplPolarity::Positive) |
                    (ast::Unsafety::Normal, ast::Unsafety::Normal, _) => {
                        /* OK */
                    }
                }
            }
        }
    }
}

impl<'cx, 'tcx,'v> visit::Visitor<'v> for UnsafetyChecker<'cx, 'tcx> {
    fn visit_item(&mut self, item: &'v ast::Item) {
        match item.node {
            ast::ItemDefaultImpl(unsafety, _) => {
                self.check_unsafety_coherence(item, unsafety, ast::ImplPolarity::Positive);
            }
            ast::ItemImpl(unsafety, polarity, _, _, _, _) => {
                self.check_unsafety_coherence(item, unsafety, polarity);
            }
            _ => { }
        }

        visit::walk_item(self, item);
    }
}
