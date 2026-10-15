import { bindable } from 'aurelia';
import {
  charCount,
  limits,
  slugify,
  validateDraft,
  type Field,
  type FieldError,
  type Limits,
  type NoteDraft,
} from '../core';
import { InvalidNoteError } from '../notes-api';

export class NoteForm {
  /** Called with a valid draft. Rejecting keeps the form filled in. */
  @bindable save!: (draft: NoteDraft) => Promise<void>;

  title = '';
  body = '';
  isSaving = false;
  failure = '';
  readonly limits: Limits = limits();

  /** Hides "required" errors until the first submit attempt. */
  private attempted = false;

  // Every getter below calls into Rust. Aurelia tracks which properties a
  // getter reads, so these recompute as the user types.

  get errors(): FieldError[] {
    return validateDraft({ title: this.title, body: this.body });
  }

  get titleError(): string | undefined {
    return this.visibleError('title');
  }

  get bodyError(): string | undefined {
    return this.visibleError('body');
  }

  get titleLength(): number {
    return charCount(this.title.trim());
  }

  get bodyLength(): number {
    return charCount(this.body.trim());
  }

  get slug(): string {
    return slugify(this.title);
  }

  async submit(): Promise<void> {
    this.attempted = true;
    if (this.isSaving || this.errors.length > 0) {
      return;
    }

    this.isSaving = true;
    this.failure = '';
    try {
      await this.save({ title: this.title, body: this.body });
      this.title = '';
      this.body = '';
      this.attempted = false;
    } catch (error) {
      this.failure = error instanceof InvalidNoteError
        ? error.message
        : 'The note was not saved. Check that the API server is running.';
    } finally {
      this.isSaving = false;
    }
  }

  private visibleError(field: Field): string | undefined {
    if (!this.attempted && this[field] === '') {
      return undefined;
    }
    return this.errors.find(error => error.field === field)?.message;
  }
}
