/**
 * What the navigation shows: the figures behind each workflow step (read from the same
 * commands the pages use) and which jobs are running. In memory only. The figures are
 * refreshed when the project, the page or a running job changes.
 */
import { classifyImages, listAdjustments, listExports, listSamplingRuns, projectSummary } from '../ipc/app';
import type { Figures, Running } from '../pipeline';
import { adjustJob } from './adjust.svelte';
import { consolidationJob } from './consolidate.svelte';
import { sheetJob } from './contactSheet.svelte';
import { exportJob } from './export.svelte';

let figures = $state<Figures | null>(null);
/** The analysis runs inside the Cleaning page; it reports here so the nav can show it. */
let analyzing = $state(false);
let seq = 0;

export const pipeline = {
  get figures(): Figures | null {
    return figures;
  },
  setAnalyzing(on: boolean): void {
    analyzing = on;
  },
  get running(): Running {
    return {
      analysis: analyzing,
      adjust: adjustJob.state.running,
      export: exportJob.state.exporting,
      consolidation: consolidationJob.state.busy,
      sheet: sheetJob.state.running
    };
  },
  clear(): void {
    seq++;
    figures = null;
  },
  /** Re-read the counts. A failure keeps the previous figures (the nav is not the place for errors). */
  async refresh(root: string): Promise<void> {
    if (!root) {
      this.clear();
      return;
    }
    const mine = ++seq;
    try {
      const [summary, counts, runs, exports, adjustments] = await Promise.all([
        projectSummary(root),
        classifyImages(root),
        listSamplingRuns(root),
        listExports(root),
        listAdjustments(root)
      ]);
      if (mine !== seq) return;
      figures = {
        photos: summary.total_images,
        unanalyzed: counts.unanalyzed,
        flagged: counts.flagged,
        selections: runs.length,
        adjustments: adjustments.flicker.length + adjustments.stabilization.length,
        exports: exports.filter((e) => e.status === 'completed').length
      };
    } catch {
      /* keep what we had */
    }
  }
};
