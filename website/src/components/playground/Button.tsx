// The host's native Button — the component that fills the `hostSlot("Button")`
// hole punched by the `action` recipe. Plugins only declare intent; this app
// decides what a button looks like (and shows which plugin a click targets).
import type { ActionRef, HostComponentProps } from 'unode-react';

import { text } from './recipes';

export function PluginButton({ children, intent = 'secondary', action, dispatch }: HostComponentProps) {
	const label = text(children);
	const actionRef = action as ActionRef | undefined;
	return (
		<button
			className={`pg-button intent-${text(intent) || 'secondary'}`}
			type="button"
			title={actionRef?.originPluginId ? `Dispatches to ${actionRef.originPluginId}` : undefined}
			onClick={() => actionRef && dispatch(actionRef)}
		>
			<span>{label}</span>
			{actionRef?.originPluginId && <small>{actionRef.originPluginId.split('.').pop()}</small>}
		</button>
	);
}
