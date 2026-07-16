// The host's native button — in vanilla there is no component framework, so
// the "Button" is simply an `action` recipe that produces a real DOM button
// via `h()`. No host-slot portal needed: this is the vanilla idiom.
import { h, type Recipe } from "unode-web-renderer";

type ButtonIntent = "primary" | "secondary" | "ghost" | "danger";

function normalizeIntent(intent: unknown): ButtonIntent {
  return intent === "primary" || intent === "secondary" || intent === "ghost" || intent === "danger"
    ? intent
    : "secondary";
}

export const buttonRecipe: Recipe = ({ label, prop, run }) => {
  const intent = normalizeIntent(prop("intent"));
  const disabled = prop("disabled") === true;
  return h(
    "button",
    {
      class: `ds-button ds-button--${intent}`,
      disabled,
      onClick: run,
      "data-intent": intent,
    },
    label,
  );
};
