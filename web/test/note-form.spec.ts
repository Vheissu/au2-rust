import { describe, expect, it, vi } from 'vitest';
import { createFixture } from '@aurelia/testing';
import { tasksSettled } from '@aurelia/runtime';
import { NoteForm } from '../src/components/note-form';
import { InvalidNoteError } from '../src/notes-api';
import type { NoteDraft } from '../src/core';

function render(save: (draft: NoteDraft) => Promise<void> = async () => {}) {
  return createFixture(
    '<note-form save.bind="save"></note-form>',
    { save },
    [NoteForm],
  ).started;
}

describe('note-form', () => {
  it('shows the slug the server will assign', async () => {
    const { type, assertText } = await render();

    type('#note-title', 'Hello, Wörld!');
    await tasksSettled();

    assertText('#note-title-hint', 'Saved at /notes/hello-wörld', { compact: true });
  });

  it('counts characters the way Rust does', async () => {
    const { type, assertText } = await render();

    // JavaScript says '🦀'.length is 2. The shared rules count it once.
    type('#note-title', 'a🦀');
    await tasksSettled();

    assertText('#note-title-count', '2 / 80', { compact: true });
  });

  it('blocks an invalid draft without calling save', async () => {
    const save = vi.fn(async () => {});
    const { trigger, assertText } = await render(save);

    trigger('form', 'submit');
    await tasksSettled();

    expect(save).not.toHaveBeenCalled();
    assertText('#note-title-error', 'Title is required.');
  });

  it('saves a valid draft and clears the form', async () => {
    const save = vi.fn(async () => {});
    const { type, trigger, assertValue } = await render(save);

    type('#note-title', 'Groceries');
    type('#note-body', 'Milk');
    trigger('form', 'submit');

    // save() is async, so wait for the form to react to it settling.
    await vi.waitFor(() => assertValue('#note-title', ''));
    expect(save).toHaveBeenCalledWith({ title: 'Groceries', body: 'Milk' });
  });

  it('shows server-side validation errors', async () => {
    const save = vi.fn(async () => {
      throw new InvalidNoteError([{ field: 'title', message: 'Taken.' }]);
    });
    const { type, trigger, assertText, assertValue } = await render(save);

    type('#note-title', 'Groceries');
    trigger('form', 'submit');

    await vi.waitFor(() => assertText('[role=alert]', 'Taken.'));
    assertValue('#note-title', 'Groceries');
  });
});
