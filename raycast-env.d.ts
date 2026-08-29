/// <reference types="@raycast/api">

/* 🚧 🚧 🚧
 * This file is auto-generated from the extension's manifest.
 * Do not modify manually. Instead, update the `package.json` file.
 * 🚧 🚧 🚧 */

/* eslint-disable @typescript-eslint/ban-types */

type ExtensionPreferences = {}

/** Preferences accessible in all the extension's commands */
declare type Preferences = ExtensionPreferences

declare namespace Preferences {
  /** Preferences accessible in the `convert` command */
  export type Convert = ExtensionPreferences & {
  /** Base PX Value - The conversion will be calculated based on this property. Default 16px */
  "basePixel"?: string
}
}

declare namespace Arguments {
  /** Arguments passed to the `convert` command */
  export type Convert = {}
}

