import { resolve } from 'aurelia';
import type { Note, NoteDraft } from './core';
import { INotesApi } from './notes-api';

export class MyApp {
  private readonly api = resolve(INotesApi);

  notes: Note[] = [];
  loadError = '';

  async binding(): Promise<void> {
    try {
      this.notes = await this.api.list();
    } catch {
      this.loadError = 'Could not reach the API. Start it with cargo run from the repository root.';
    }
  }

  async add(draft: NoteDraft): Promise<void> {
    const note = await this.api.create(draft);
    this.notes.push(note);
    this.loadError = '';
  }

  async remove(note: Note): Promise<void> {
    await this.api.remove(note.id);
    this.notes = this.notes.filter(item => item.id !== note.id);
  }
}
