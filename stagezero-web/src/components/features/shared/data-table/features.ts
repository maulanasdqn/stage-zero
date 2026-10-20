import {
  rowPaginationFeature,
  rowSortingFeature,
  tableFeatures,
  type Column,
  type ColumnDef,
  type RowData,
} from '@tanstack/react-table'

export const dataTableFeatures = tableFeatures({ rowSortingFeature, rowPaginationFeature })

export type TDataTableFeatures = typeof dataTableFeatures

export type TColumnDef<TData extends RowData> = ColumnDef<TDataTableFeatures, TData>

export type TColumn<TData extends RowData, TValue = unknown> = Column<
  TDataTableFeatures,
  TData,
  TValue
>
