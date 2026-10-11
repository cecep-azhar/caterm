// CAUI Bridge - backward-compatible re-exports from @cecepazhar/caui
// ponytail: this barrel replaces direct $lib/components/ui/ imports during migration; remove once all consumers switch to @cecepazhar/caui directly.

export {
  Button,
  Badge,
  Card,
  Icon,
  Modal,
  Input,
  Alert,
  Table,
  LanguageSwitcher,
  ThemeSwitcher,
  Sidebar,
  SidebarItem,
} from '@cecepazhar/caui';

// Local type aliases exported for consumer compatibility.
// Re-exported from the thin wrappers so existing import paths work.
export type {
  ButtonVariant,
  ButtonSize,
  BadgeVariant,
  BadgeSize,
  IconName,
  AlertVariant,
  Column,
  BrandAccent,
} from './index';
