// src/utils/parseCSV.ts
import Papa from 'papaparse'

export interface ParseCSVResult {
  headers: string[]
  data: any[]
}

export const parseCSV = async (file: File): Promise<ParseCSVResult> => {
  return new Promise((resolve, reject) => {
    Papa.parse(file, {
      header: true,
      dynamicTyping: true,
      skipEmptyLines: true,
      transform: (value: any, field: string) => {
        // Handle MongoDB-specific format conversions
        if (field.toLowerCase() === '_id') {
          return { $oid: value.toString() }
        }
        if (value instanceof Date) {
          return value.toISOString()
        }
        return value
      },
      complete: (results) => {
        if (results.errors.length) {
          return reject(new Error('CSV parsing errors: ' + JSON.stringify(results.errors)))
        }
        resolve({
          headers: results.meta.fields || [],
          data: results.data,
        })
      },
      error: (error: Error) => reject(error),
    })
  })
}
